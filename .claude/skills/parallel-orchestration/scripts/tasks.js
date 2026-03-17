#!/usr/bin/env node
/**
 * tasks.yaml / status.yaml の参照・更新を行う
 *
 * 参照系:
 *   node tasks.js active                                          # active task のリスト
 *   node tasks.js pending                                         # pending task のリスト
 *   node tasks.js completed                                       # completed task のリスト
 *   node tasks.js all                                             # 全タスクのリスト
 *   node tasks.js unblocked                                       # 依存が解消された pending task のリスト
 *   node tasks.js worktreePath <task-id>                          # active task の worktree パスを取得
 *   node tasks.js branch <task-id>                                # active task のブランチ名を取得
 *
 * 更新系:
 *   node tasks.js toActive <task-id> <worktree-path> <branch> <pid>   # pending → active に移動
 *   node tasks.js toCompleted <task-id>                                # active → completed に移動
 *   node tasks.js toFailed <task-id> <error-message>                   # active → failed に記録
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

// ============================================================
// 簡易 YAML パーサー（基本的な構造のみ対応）
// ============================================================

function parseSimpleYaml(content) {
  const result = {};
  const lines = content.split('\n');
  let currentKey = null;
  let inArrayOfObjects = false;
  let currentObject = null;

  for (const line of lines) {
    const trimmed = line.trimEnd();

    // 空行やコメントはスキップ
    if (!trimmed || trimmed.startsWith('#')) continue;

    const indent = line.length - line.trimStart().length;

    // 配列アイテム（オブジェクト）
    if (trimmed.startsWith('- ') && trimmed.includes(':')) {
      if (currentKey && Array.isArray(result[currentKey])) {
        const match = trimmed.match(/^-\s+(\w+):\s*(.*)$/);
        if (match) {
          currentObject = { [match[1]]: match[2].replace(/^["']|["']$/g, '') };
          result[currentKey].push(currentObject);
          inArrayOfObjects = true;
        }
      }
      continue;
    }

    // オブジェクト内のプロパティ
    if (inArrayOfObjects && indent > 2 && currentObject) {
      const match = trimmed.match(/^(\w+):\s*(.*)$/);
      if (match) {
        let value = match[2].replace(/^["']|["']$/g, '');
        // 配列値の処理
        if (value.startsWith('[') && value.endsWith(']')) {
          value = value.slice(1, -1).split(',').map(s => s.trim().replace(/^["']|["']$/g, '')).filter(Boolean);
        }
        currentObject[match[1]] = value;
      }
      continue;
    }

    // 単純な配列アイテム
    if (trimmed.startsWith('- ')) {
      if (currentKey && Array.isArray(result[currentKey])) {
        result[currentKey].push(trimmed.slice(2).replace(/^["']|["']$/g, ''));
      }
      continue;
    }

    // キー: 値 または キー:
    const match = trimmed.match(/^(\w+):\s*(.*)$/);
    if (match) {
      inArrayOfObjects = false;
      currentObject = null;
      currentKey = match[1];
      const value = match[2].replace(/^["']|["']$/g, '');

      if (value === '' || value === '[]') {
        result[currentKey] = [];
      } else if (value.startsWith('[') && value.endsWith(']')) {
        result[currentKey] = value.slice(1, -1).split(',').map(s => s.trim().replace(/^["']|["']$/g, '')).filter(Boolean);
      } else {
        result[currentKey] = value;
      }
    }
  }

  return result;
}

// ============================================================
// status.yaml シリアライザー
// ============================================================

function serializeStatusYaml(status) {
  const lines = [];

  lines.push(`status: ${status.status}`);
  lines.push(`feature_branch: ${status.feature_branch}`);
  lines.push(`started_at: ${status.started_at}`);
  lines.push(`updated_at: ${status.updated_at}`);
  lines.push('');

  // active_tasks
  const activeTasks = status.active_tasks || [];
  if (activeTasks.length === 0) {
    lines.push('active_tasks: []');
  } else {
    lines.push('active_tasks:');
    for (const task of activeTasks) {
      lines.push(`  - task_id: ${task.task_id}`);
      lines.push(`    worktree_path: ${task.worktree_path}`);
      lines.push(`    branch: ${task.branch}`);
      lines.push(`    worker_pid: ${task.worker_pid}`);
      lines.push(`    started_at: ${task.started_at}`);
    }
  }
  lines.push('');

  // completed_tasks
  const completedTasks = status.completed_tasks || [];
  if (completedTasks.length === 0) {
    lines.push('completed_tasks: []');
  } else {
    lines.push('completed_tasks:');
    for (const id of completedTasks) {
      lines.push(`  - ${id}`);
    }
  }
  lines.push('');

  // pending_tasks
  const pendingTasks = status.pending_tasks || [];
  if (pendingTasks.length === 0) {
    lines.push('pending_tasks: []');
  } else {
    lines.push('pending_tasks:');
    for (const id of pendingTasks) {
      lines.push(`  - ${id}`);
    }
  }

  // failed fields
  if (status.failed_task) {
    lines.push('');
    lines.push(`failed_task: ${status.failed_task}`);
    lines.push(`error_message: ${status.error_message || ''}`);
  }

  return lines.join('\n') + '\n';
}

// ============================================================
// 共通ヘルパー
// ============================================================

const scriptDir = __dirname;
const commonDir = path.join(scriptDir, 'common');

function runCommon(script) {
  return execSync(`"${path.join(commonDir, script)}"`, { encoding: 'utf8' }).trim();
}

function nowISO() {
  return new Date().toISOString().replace(/\.\d{3}Z$/, '+00:00');
}

// ============================================================
// パス解決
// ============================================================

let repoRoot;
try {
  repoRoot = runCommon('get-repo-root.sh');
} catch {
  process.exit(0);
}

let featureId;
try {
  featureId = runCommon('get-active-feature-id.sh');
} catch {
  process.exit(0);
}

if (!featureId) {
  process.exit(0);
}

const featureDir = path.join(repoRoot, '.agents', 'features', featureId);
const statusPath = path.join(featureDir, 'status.yaml');
const tasksPath = path.join(featureDir, 'tasks.yaml');

// ============================================================
// 参照系コマンド
// ============================================================

function queryTasks(query) {
  let tasks = [];

  switch (query) {
    case 'active': {
      const statusYaml = parseSimpleYaml(fs.readFileSync(statusPath, 'utf8'));
      const activeTasks = statusYaml.active_tasks || [];
      tasks = activeTasks.map(t => typeof t === 'string' ? t : t.task_id);
      break;
    }
    case 'pending': {
      const statusYaml = parseSimpleYaml(fs.readFileSync(statusPath, 'utf8'));
      tasks = statusYaml.pending_tasks || [];
      break;
    }
    case 'completed': {
      const statusYaml = parseSimpleYaml(fs.readFileSync(statusPath, 'utf8'));
      tasks = statusYaml.completed_tasks || [];
      break;
    }
    case 'all': {
      const tasksYaml = parseSimpleYaml(fs.readFileSync(tasksPath, 'utf8'));
      const taskList = tasksYaml.tasks || [];
      tasks = taskList.map(t => typeof t === 'string' ? t : t.id);
      break;
    }
    case 'unblocked': {
      const statusYaml = parseSimpleYaml(fs.readFileSync(statusPath, 'utf8'));
      const tasksYaml = parseSimpleYaml(fs.readFileSync(tasksPath, 'utf8'));

      const completed = new Set(statusYaml.completed_tasks || []);
      const pending = new Set(statusYaml.pending_tasks || []);
      const taskList = tasksYaml.tasks || [];

      tasks = taskList
        .filter(t => pending.has(t.id))
        .filter(t => {
          const deps = t.dependencies || [];
          return deps.length === 0 || deps.every(d => completed.has(d));
        })
        .map(t => t.id);
      break;
    }
    default:
      return null;
  }

  return tasks;
}

// ============================================================
// 更新系コマンド
// ============================================================

/**
 * toActive: pending → active に移動
 * args: [task-id, worktree-path, branch, pid]
 */
function activateTask(args) {
  if (args.length < 4) {
    console.error('Usage: tasks.js toActive <task-id> <worktree-path> <branch> <pid>');
    process.exit(1);
  }

  const [taskId, worktreePath, branch, pid] = args;
  const status = parseSimpleYaml(fs.readFileSync(statusPath, 'utf8'));

  // active_tasks に追加
  if (!Array.isArray(status.active_tasks)) {
    status.active_tasks = [];
  }
  status.active_tasks.push({
    task_id: taskId,
    worktree_path: worktreePath,
    branch: branch,
    worker_pid: pid,
    started_at: nowISO(),
  });

  // pending_tasks から削除
  if (Array.isArray(status.pending_tasks)) {
    status.pending_tasks = status.pending_tasks.filter(id => id !== taskId);
  }

  status.updated_at = nowISO();

  fs.writeFileSync(statusPath, serializeStatusYaml(status));
}

/**
 * toCompleted: active → completed に移動
 * args: [task-id]
 */
function completeTask(args) {
  if (args.length < 1) {
    console.error('Usage: tasks.js toCompleted <task-id>');
    process.exit(1);
  }

  const [taskId] = args;
  const status = parseSimpleYaml(fs.readFileSync(statusPath, 'utf8'));

  // active_tasks から削除
  if (Array.isArray(status.active_tasks)) {
    status.active_tasks = status.active_tasks.filter(t =>
      (typeof t === 'string' ? t : t.task_id) !== taskId
    );
  }

  // completed_tasks に追加
  if (!Array.isArray(status.completed_tasks)) {
    status.completed_tasks = [];
  }
  if (!status.completed_tasks.includes(taskId)) {
    status.completed_tasks.push(taskId);
  }

  // 全タスク完了チェック
  const activeTasks = status.active_tasks || [];
  const pendingTasks = status.pending_tasks || [];
  if (activeTasks.length === 0 && pendingTasks.length === 0) {
    status.status = 'completed';
  }

  status.updated_at = nowISO();

  fs.writeFileSync(statusPath, serializeStatusYaml(status));
}

/**
 * toFailed: active → failed に記録
 * args: [task-id, error-message]
 */
function failTask(args) {
  if (args.length < 1) {
    console.error('Usage: tasks.js toFailed <task-id> [error-message]');
    process.exit(1);
  }

  const taskId = args[0];
  const errorMessage = args.slice(1).join(' ') || 'Unknown error';
  const status = parseSimpleYaml(fs.readFileSync(statusPath, 'utf8'));

  // active_tasks から削除
  if (Array.isArray(status.active_tasks)) {
    status.active_tasks = status.active_tasks.filter(t =>
      (typeof t === 'string' ? t : t.task_id) !== taskId
    );
  }

  status.status = 'failed';
  status.failed_task = taskId;
  status.error_message = errorMessage;
  status.updated_at = nowISO();

  fs.writeFileSync(statusPath, serializeStatusYaml(status));
}

// ============================================================
// メイン
// ============================================================

const command = process.argv[2];
if (!command) {
  console.error('Usage: tasks.js <command> [args...]');
  console.error('');
  console.error('参照系: active | pending | completed | all | unblocked');
  console.error('更新系: toActive <task-id> <worktree-path> <branch> <pid>');
  console.error('        toCompleted <task-id>');
  console.error('        toFailed <task-id> [error-message]');
  process.exit(1);
}

try {
  const commandArgs = process.argv.slice(3);

  switch (command) {
    // 更新系
    case 'toActive':
      activateTask(commandArgs);
      break;
    case 'toCompleted':
      completeTask(commandArgs);
      break;
    case 'toFailed':
      failTask(commandArgs);
      break;

    // 参照系
    case 'worktreePath': {
      // worktreePath <task-id>: active_tasks から worktree_path を取得
      const taskId = commandArgs[0];
      if (!taskId) {
        console.error('Usage: tasks.js worktreePath <task-id>');
        process.exit(1);
      }
      const status = parseSimpleYaml(fs.readFileSync(statusPath, 'utf8'));
      const activeTasks = status.active_tasks || [];
      const found = activeTasks.find(t => (typeof t === 'string' ? t : t.task_id) === taskId);
      if (found && found.worktree_path) {
        console.log(found.worktree_path);
      } else {
        console.error(`Task not found in active_tasks: ${taskId}`);
        process.exit(1);
      }
      break;
    }
    case 'branch': {
      // branch <task-id>: active_tasks から branch を取得
      const taskId2 = commandArgs[0];
      if (!taskId2) {
        console.error('Usage: tasks.js branch <task-id>');
        process.exit(1);
      }
      const status2 = parseSimpleYaml(fs.readFileSync(statusPath, 'utf8'));
      const activeTasks2 = status2.active_tasks || [];
      const found2 = activeTasks2.find(t => (typeof t === 'string' ? t : t.task_id) === taskId2);
      if (found2 && found2.branch) {
        console.log(found2.branch);
      } else {
        console.error(`Task not found in active_tasks: ${taskId2}`);
        process.exit(1);
      }
      break;
    }

    default: {
      const tasks = queryTasks(command);
      if (tasks === null) {
        console.error(`Unknown command: ${command}`);
        process.exit(1);
      }
      if (tasks.length > 0) {
        console.log(tasks.join('\n'));
      }
      break;
    }
  }
} catch (e) {
  // ファイルが存在しない場合など
  if (e.code === 'ENOENT') {
    process.exit(0);
  }
  console.error(e.message);
  process.exit(1);
}

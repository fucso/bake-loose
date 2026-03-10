#!/usr/bin/env node
/**
 * tasks.yaml / status.yaml からタスクリストを取得する
 *
 * 引数: query (active | pending | completed | all | unblocked)
 * 出力: タスク ID（改行区切り）
 *
 * 使用例:
 *   node tasks.js active     # active task のリスト
 *   node tasks.js pending    # pending task のリスト
 *   node tasks.js completed  # completed task のリスト
 *   node tasks.js all        # 全タスクのリスト
 *   node tasks.js unblocked  # 依存が解消された pending task のリスト
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

// 簡易 YAML パーサー（基本的な構造のみ対応）
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

// common スクリプト経由で値を取得
const scriptDir = __dirname;
const commonDir = path.join(scriptDir, 'common');

function runCommon(script) {
  return execSync(`"${path.join(commonDir, script)}"`, { encoding: 'utf8' }).trim();
}

// メイン処理
const query = process.argv[2];
if (!query) {
  console.error('Usage: tasks.js <active|pending|completed|all|unblocked>');
  process.exit(1);
}

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

let tasks = [];

try {
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
      console.error(`Unknown query: ${query}`);
      process.exit(1);
  }
} catch (e) {
  // ファイルが存在しない場合など
  process.exit(0);
}

if (tasks.length > 0) {
  console.log(tasks.join('\n'));
}

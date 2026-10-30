import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';
import { build } from 'esbuild';
import { parse } from '@babel/parser';
const zh = JSON.parse(fs.readFileSync('src/locales/zh-CN.json', 'utf8'));
const en = JSON.parse(fs.readFileSync('src/locales/en.json', 'utf8'));
assert.deepEqual(Object.keys(zh).sort(), Object.keys(en).sort(), 'locale key parity');
const placeholders = s => [...s.matchAll(/\{(\w+)\}/g)].map(m => m[1]).sort();
for (const key of Object.keys(zh)) {
  assert.ok(en[key].trim(), `Missing English: ${key}`);
  assert.deepEqual(placeholders(zh[key]), placeholders(en[key]), `Placeholders: ${key}`);
}
const source = fs.readFileSync('src/main.tsx', 'utf8');
const ast = parse(source, {sourceType:'module', plugins:['typescript','jsx']});
function walk(node) {
  if (node.type === 'CallExpression' && node.callee.name === 't' && node.arguments[0]?.type === 'StringLiteral') {
    assert.ok(Object.hasOwn(zh, node.arguments[0].value), `Unknown translation key: ${node.arguments[0].value}`);
  }
  if (node.type === 'JSXText' && /\p{Script=Han}/u.test(node.value)) {
    assert.equal(node.value.trim(), '简体中文', 'Untranslated JSX text');
  }
  for (const v of Object.values(node)) {
    if (Array.isArray(v)) v.forEach(n => n && typeof n === 'object' && walk(n));
    else if(v && typeof v === 'object' && v.type) walk(v);
  }
}
walk(ast);
const bundled = await build({entryPoints:['src/i18n.ts'], bundle:true, platform:'node', format:'cjs', write:false});
const context = {module:{exports:{}}};
vm.runInNewContext(bundled.outputFiles[0].text, context);
const {translate:t, diagnostic:d, resolveLocale:r} = context.module.exports;
assert.equal(r('system','zh-Hant-TW'), 'zh-CN');
assert.equal(r('system','de-DE'), 'en');
assert.equal(r('en','zh-CN'), 'en');
assert.equal(r('zh-CN','en-US'), 'zh-CN');
assert.equal(t('en','删除 {name}', {name:'中文 app ${port}'}), 'Remove 中文 app ${port}');
assert.equal(t('zh-CN','删除 {name}', {name:'Demo'}), '删除 Demo');
assert.ok(t('en','使用前台常驻命令。支持 ${port} 占位符；现有服务的依赖需已准备好。').includes('${port}'));
assert.equal(t('en','unknown'), 'unknown');
assert.equal(d('en','toString'), 'toString');
assert.equal(d('en','HTTP 检查失败'), 'HTTP check failed');
assert.equal(d('en','安装步骤 2/3'), 'Installation step 2/3');
const conflict='端口 3000 已被占用：node（PID 42，IPv6 *:3000）。请先停止占用进程，或修改此服务的端口后再启动。';
assert.ok(d('en',conflict).startsWith('Port 3000 is already in use'));
assert.ok(d('en',conflict).includes('PID 42'));
assert.ok(d('en',`启动失败（exit status: 1）：${conflict}`).startsWith('Startup failed (exit status: 1): Port 3000'));
assert.equal(d('en','操作已取消：安装已取消'),'Operation cancelled: Installation cancelled');
assert.equal(d('en','安装步骤 1/2失败（exit status: 7），请查看日志'), 'Installation step 1/2 failed (exit status: 7); see the logs');
assert.equal(d('en','不支持的占位符：${unknown}'), 'Unsupported placeholder: ${unknown}');
assert.equal(d('en','raw unknown 中文 error'), 'raw unknown 中文 error');
assert.equal(d('zh-CN',conflict), conflict);
const catalog=JSON.parse(fs.readFileSync('registry/mineru.json','utf8'));
for(const value of [catalog.description,...catalog.releases.flatMap(r=>[r.label,r.notes])]) {
  if(/\p{Script=Han}/u.test(value)) assert.ok(Object.hasOwn(en,value), `Untranslated catalog: ${value}`);
}
console.log(`i18n checks passed: ${Object.keys(en).length} bilingual messages, interpolation, diagnostics, fallback, JSX, and catalog coverage.`);

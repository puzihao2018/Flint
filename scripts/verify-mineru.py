#!/usr/bin/env python3
"""Validate local MinerU 4.1 Tasks API using supplied synthetic documents."""
import argparse, io, json, pathlib, time, urllib.request, urllib.parse, zipfile
p=argparse.ArgumentParser();p.add_argument('--base',default='http://127.0.0.1:8000');p.add_argument('--out',required=True);p.add_argument('--tier',default='standard');p.add_argument('files',nargs='+');a=p.parse_args()
out=pathlib.Path(a.out);out.mkdir(parents=True,exist_ok=True)
http=urllib.request.build_opener(urllib.request.ProxyHandler({}))
def request(url,method='GET',body=None,headers=None):
    if urllib.parse.urlsplit(url).netloc!=urllib.parse.urlsplit(a.base).netloc:raise RuntimeError('Unexpected nonlocal result/upload URL')
    with http.open(urllib.request.Request(url,data=body,method=method,headers=headers or {}),timeout=30) as r:return r.read()
def api(path,method='GET',body=None):return json.loads(request(a.base+path,method,None if body is None else json.dumps(body).encode(),{'Content-Type':'application/json'}))
health=api('/v1/health');models=api('/v1/models');assert health['status']=='ok'
paths=[pathlib.Path(f) for f in a.files]
created=api('/v1/tasks','POST',{'files':[{'file_name':f.name,'bytes':f.stat().st_size} for f in paths],'tier':a.tier,'ocr_mode':'auto','output_formats':['default']})
(out/'created.json').write_text(json.dumps(created,indent=2));print('Created',created['task_id'],flush=True)
for f,receipt in zip(paths,created['files']):
    if 'upload' in receipt:request(receipt['upload']['url'],'PUT',f.read_bytes(),receipt['upload']['headers'])
start=time.monotonic();previous=''
while time.monotonic()-start<600:
    task=api('/v1/tasks/'+created['task_id']);state=json.dumps([(f['file_name'],f['status']) for f in task['files']])
    if state!=previous:print(state,flush=True);previous=state
    if task['status'] in ('completed','failed','partial','canceled'):break
    time.sleep(2)
else:raise RuntimeError('Parse exceeded 600 seconds')
(out/'task.json').write_text(json.dumps(task,indent=2));assert task['status']=='completed',task
reports=[]
for f in task['files']:
    assert f['status']=='completed',f
    result=request(f['result_url']);(out/(f['file_name']+'.zip')).write_bytes(result)
    with zipfile.ZipFile(io.BytesIO(result)) as z:
        md_name=next(n for n in z.namelist() if n.endswith('markdown.md'));md=z.read(md_name).decode();(out/(f['file_name']+'.md')).write_text(md)
        assert 'flint' in md.lower() and '7319' in md,md
        assert any(n.endswith('structured_content.json') for n in z.namelist())
    reports.append({'file':f['file_name'],'parse':f.get('parse'),'verifiedText':['Flint','7319'],'markdown':md,'zipBytes':len(result)})
report={'base':a.base,'elapsedSeconds':round(time.monotonic()-start,2),'health':health,'models':models,'results':reports}
(out/'report.json').write_text(json.dumps(report,indent=2));print(json.dumps(report,indent=2),flush=True)

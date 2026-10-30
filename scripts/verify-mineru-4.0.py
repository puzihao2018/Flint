#!/usr/bin/env python3
"""Exercise the upload/Jobs/ZIP contract used by llm-for-zotero 3.9.15."""
import argparse, io, json, pathlib, time, urllib.request, urllib.parse, zipfile
p=argparse.ArgumentParser();p.add_argument('--base',default='http://127.0.0.1:8001');p.add_argument('--out',required=True);p.add_argument('files',nargs='+');a=p.parse_args()
out=pathlib.Path(a.out);out.mkdir(parents=True,exist_ok=True)
http=urllib.request.build_opener(urllib.request.ProxyHandler({}))
def request(path,method='GET',body=None,headers=None):
    url=urllib.parse.urljoin(a.base+'/',path)
    if urllib.parse.urlsplit(url).netloc!=urllib.parse.urlsplit(a.base).netloc:raise RuntimeError('Unexpected external URL')
    with http.open(urllib.request.Request(url,data=body,method=method,headers=headers or {}),timeout=60) as r:return r.read()
def api(path,body=None,method=None):return json.loads(request(path,method or ('POST' if body is not None else 'GET'),None if body is None else json.dumps(body).encode(),{'Content-Type':'application/json'}))
health=api('/v1/health');assert health['version']=='4.0.12';assert 'file_id' in health['features']['sources'];assert 'zip' in health['features']['output_formats']
tiers=api('/v1/tiers');(out/'tiers.json').write_text(json.dumps(tiers,indent=2));print('Health and legacy capabilities passed',flush=True)
reports=[]
for filename in a.files:
    f=pathlib.Path(filename);start=time.monotonic()
    upload=api('/v1/uploads',{'filename':f.name,'bytes':f.stat().st_size,'mime_type':'application/pdf','purpose':'parse'})
    if upload['status']=='pending':
        assert upload['upload_method']=='PUT'
        request(upload['upload_url'],'PUT',f.read_bytes(),upload['upload_headers'])
        upload=api('/v1/uploads/'+upload['id']+'/complete',{})
    assert upload['status']=='completed'
    file_id=upload['file']['id']
    job=api('/v1/parse/jobs',{'files':[{'source':{'type':'file_id','file_id':file_id},'page_range':'all'}],'tier':'standard','ocr_mode':'auto','output_formats':['zip']})
    job_path='/v1/parse/jobs/'+job['job_id'];previous=''
    while time.monotonic()-start<600:
        if job['status']!=previous:print(f.name,job['status'],flush=True);previous=job['status']
        if job['status'] in ('completed','failed','partial','canceled'):break
        time.sleep(2);job=api(job_path)
    (out/(f.name+'.job.json')).write_text(json.dumps(job,indent=2));assert job['status']=='completed',job
    result=job['files'][0];assert result['status']=='completed' and result['file_id']==file_id
    payload=request('/v1/files/'+result['output_files']['zip']['file_id']+'/content')
    (out/(f.name+'.zip')).write_bytes(payload)
    with zipfile.ZipFile(io.BytesIO(payload)) as z:
        names=z.namelist();assert 'markdown.md' in names and 'structured_content.json' in names
        structured=json.loads(z.read('structured_content.json'));assert structured['pages'] and structured.get('is_full_document') is not False
        for index,page in enumerate(structured['pages']):
            assert page['page_idx']==index and isinstance(page['blocks'],list)
            for block in page['blocks']:
                assert isinstance(block['type'],str) and isinstance(block['content'],str)
                if block.get('image_source') is not None:assert block['image_source'] in names
        md=z.read('markdown.md').decode()
        assert 'flint' in md.lower() and '7319' in md,md
        (out/(f.name+'.md')).write_text(md)
    reports.append({'file':f.name,'seconds':round(time.monotonic()-start,2),'verifiedText':['Flint','7319'],'zipBytes':len(payload),'markdown':md})
report={'base':a.base,'health':health,'contract':'llm-for-zotero 3.9.15 upload/Jobs/ZIP','results':reports}
(out/'report.json').write_text(json.dumps(report,indent=2));print(json.dumps(report,indent=2),flush=True)

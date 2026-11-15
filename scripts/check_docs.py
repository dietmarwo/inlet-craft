#!/usr/bin/env python3
"""Execute authored documentation fences; generate schemas/reference/static site.
Upstream vendor docs and verbatim supplied design attachments are inventoried as
external inputs, not authored executable docs. No silent skip for authored fences.
"""
import html,json,os,re,subprocess
from pathlib import Path
import markdown
ROOT=Path(__file__).resolve().parents[1];CLI=ROOT/'target/release/inlet-craft'
def require(ok,message):
    if not ok:raise RuntimeError(message)
def main():
    desc=json.loads(subprocess.check_output([str(CLI),'describe'],text=True,cwd=ROOT))
    codes=set()
    for file in (ROOT/'crates').rglob('*.rs'):
        codes.update(re.findall(r'Diagnostic::new\(\s*"([EW]_[A-Z_]+)"',file.read_text()))
    require(codes.issubset(set(desc['diagnostic_codes'])),f'missing described diagnostic codes: {codes-set(desc["diagnostic_codes"])}')
    expected=[]
    for line in (ROOT/'docs/developer/formula-registry.md').read_text().splitlines():
        if line.startswith('|') and not line.startswith('|---') and not line.startswith('| ID'):
            c=[x.strip() for x in line.strip('|').split('|')]
            if len(c)==4:expected.append(dict(id=c[0],equation=c[1],source_and_validity=c[2],verification_anchor_and_limit=c[3]))
    require(desc['formulas']==expected,'embedded formula catalog stale; regenerate fixtures/formula-catalog.json before build')
    for name,value in desc['schemas'].items():
        (ROOT/f'schemas/{name}.schema.json').write_text(json.dumps(value,indent=2)+'\n')
    (ROOT/'docs/reference.md').write_text('# Generated interface reference\n\nGenerated from the current binary `describe`. Schemas are in schemas/. Registry is the authoritative source/validity ledger.\n\n| Formula | Source and declared applicability |\n|---|---|\n'+'\n'.join('|'+f['id']+'|'+f['source_and_validity']+'|' for f in desc['formulas'])+'\n\nDiagnostic codes: '+', '.join(desc['diagnostic_codes'])+'.\n')
    files=[ROOT/'README.md']+sorted(p for p in (ROOT/'docs').rglob('*.md') if 'design' not in p.parts)
    results=[]
    env=os.environ.copy();env['PYTHONPATH']=str(ROOT/'target/python')+os.pathsep+env.get('PYTHONPATH','')
    for file in files:
        text=file.read_text()
        for i,(lang,body) in enumerate(re.findall(r'```([^\n]*)\n(.*?)```',text,re.S)):
            lang=lang.strip()
            if lang in ['sh','bash']:
                p=subprocess.run(['bash','-eu','-o','pipefail','-c',body],cwd=ROOT,env=env,capture_output=True,text=True,timeout=120)
                require(p.returncode==0,f'{file}:{i}: {p.stderr[:1000]}')
            elif lang in ['python','py']:
                p=subprocess.run(['python3','-c',body],cwd=ROOT,env=env,capture_output=True,text=True,timeout=30)
                require(p.returncode==0,f'{file}:{i}: {p.stderr[:1000]}')
            elif lang=='json':json.loads(body)
            else:raise RuntimeError(f'{file}: authored fence language {lang!r} has no checker')
            results.append(dict(file=str(file.relative_to(ROOT)),block=i,language=lang,passed=True))
    # Render a complete static documentation site. No external runtime/CDN.
    nav=[];articles=[]
    for file in files:
        slug=str(file.relative_to(ROOT)).replace('/','-').replace('.md','')
        title=file.read_text().splitlines()[0].lstrip('# ')
        nav.append(f'<a href="#{html.escape(slug)}">{html.escape(title)}</a>')
        content=markdown.markdown(file.read_text(),extensions=['tables','fenced_code'])
        articles.append(f'<article id="{html.escape(slug)}">{content}</article>')
    style='body{margin:0;background:#f4f6f8;color:#17212c;font:16px/1.6 system-ui}nav{position:fixed;width:250px;height:100vh;overflow:auto;padding:20px;background:#152d40;box-sizing:border-box}nav a{display:block;color:#d6ecfa;margin:10px 0;font-size:14px}main{max-width:1050px;margin-left:270px;padding:30px}article{background:white;padding:30px;margin:0 0 30px;border-radius:10px;overflow:auto}h1{line-height:1.2}pre{padding:15px;background:#edf3f7;overflow:auto}table{border-collapse:collapse;width:100%;font-size:14px}td,th{border:1px solid #dce3e8;padding:8px;text-align:left}a{color:#066da2}@media(max-width:800px){nav{position:static;width:auto;height:auto}main{margin:0;padding:12px}article{padding:20px}}'
    site='<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>inlet-craft documentation</title><style>'+style+'</style><nav><h2 style="color:white">inlet-craft</h2>'+''.join(nav)+'</nav><main>'+''.join(articles)+'</main></html>'
    (ROOT/'site/index.html').write_text(site)
    report=dict(schema='inlet-craft.documentation-check/0.1',passed=True,authored_markdown_files=len(files),executed_blocks=results,external_design_attachments=2,vendor_docs='excluded as upstream source inputs',schemas=len(desc['schemas']),site='site/index.html')
    (ROOT/'results/documentation-check.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report,indent=2))
if __name__=='__main__':main()

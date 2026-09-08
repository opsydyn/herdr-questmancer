"""Build review configurations from current sigils and the corrected proposal."""
from pathlib import Path
import copy, json, os, re, subprocess, tempfile, tomllib

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[3]
current = tomllib.loads((Path.home() / '.config/herdr/config.toml').read_text())['ui']['sidebar']['agents']
proposal = (REPO / 'docs/design/questmancer-sidebar-09.md').read_text()
three = tomllib.loads(re.search(r'```toml\n(.*?)```', proposal, re.S)[1])['ui']['sidebar']['agents']
# Retain the class-sigil styling already requested by the user.
three['rows'][1][0] = copy.deepcopy(current['rows'][0][0])
sigil = current['rows'][0][0]
name = current['rows'][0][1]
role = three['rows'][1][1]
condition = three['rows'][1][2]
vigil, hoard = three['rows'][2]
four = {'row_gap': 1, 'rows': [[sigil, name], ['state_icon', condition, vigil], [role], [hoard]]}
configs = {'current': current, 'three': three, 'four': four}
def toml(v):
    if isinstance(v, str): return json.dumps(v, ensure_ascii=False)
    if isinstance(v, bool): return str(v).lower()
    if isinstance(v, int): return str(v)
    if isinstance(v, list): return '[' + ', '.join(map(toml, v)) + ']'
    return '{ ' + ', '.join(k + ' = ' + toml(x) for k,x in v.items()) + ' }'
receipts = []
for name, config in configs.items():
    content = '[ui.sidebar.agents]\nrow_gap = 1\nrows = [\n' + ''.join('  '+toml(row)+',\n' for row in config['rows']) + ']\n'
    (ROOT / (name+'.toml')).write_text(content)
    with tempfile.TemporaryDirectory(prefix='qm-sidebar-config-') as d:
        p=Path(d)/'herdr';p.mkdir();(p/'config.toml').write_text(content)
        result=subprocess.run(['herdr','config','check'],env={**os.environ,'XDG_CONFIG_HOME':d},capture_output=True,text=True)
        assert result.returncode == 0, result.stderr
        receipts.append({'configuration':name,'result':result.stdout.strip()})
(ROOT/'configurations.json').write_text(json.dumps(configs,ensure_ascii=False,indent=2)+'\n')
(ROOT/'validation.json').write_text(json.dumps({'herdr':'0.9.0','checks':receipts,'global_configuration_changed':False,'native_visual_acceptance':'pending'},indent=2)+'\n')
print(json.dumps(receipts))

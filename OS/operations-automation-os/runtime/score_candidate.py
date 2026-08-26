#!/usr/bin/env python3
"""Interactive illustrative scoring; a score never overrides safety gates."""
import json, sys
keys=['frequency','volume','time','error_reduction','service_gain','rule_stability','data_quality','integration','reversibility','roi','exception_simplicity','low_risk','maintainability']
data=json.load(sys.stdin)
missing=[k for k in keys if k not in data]
if missing: raise SystemExit('missing: '+','.join(missing))
for k in keys:
    if not isinstance(data[k],(int,float)) or not 0<=data[k]<=5: raise SystemExit(k+' must be 0..5')
score=sum(data[k] for k in keys)/(5*len(keys))*100
print(json.dumps({'score':round(score,1),'note':'Decision still requires evidence, risk gates and process redesign.'},indent=2))

import sys
import json
from subtitle import getvideo

def run():
    try:
        filepath=sys.argv[1] if len(sys.argv)>1 else None
        maxwords=int(sys.argv[2]) if len(sys.argv) >2 else None
        subs=getvideo(filepath,maxwords)
        res={
            "status":"ok",
            "message":subs
        }
        print(json.dumps(res,indent=4))
    except Exception as e:
        err={"status":"error","message":str(e)}
        print(json.dumps(err,indent=4))


if __name__ == "__main__":
    run()
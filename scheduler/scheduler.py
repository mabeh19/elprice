import time, datetime

def sleepUntilNext(hour, minute=0):
    t = datetime.datetime.today()
    future = datetime.datetime(t.year, t.month, t.day, t.hour + hour, minute)
    if t.timestamp() > future.timestamp():
        future += datetime.timedelta(days=1)
    time.sleep((future-t).total_seconds())

def sleepUntil(hour, minute=0):
    t = datetime.datetime.today()
    future = datetime.datetime(t.year, t.month, t.day, hour, minute)
    if t.timestamp() > future.timestamp():
        future += datetime.timedelta(days=1)
    time.sleep((future-t).total_seconds())

"""把清单里的 android.app.NativeActivity 换成我们的薄壳 MainBack（等长 26 字符）。"""
import sys

path = sys.argv[1]
raw = open(path, 'rb').read()
old = b'android.app.NativeActivity'
new = b'com.cakecake.rust.MainBack'
assert len(old) == len(new), '长度必须一致，否则会破坏字符串池偏移'
raw = raw.replace(old, new).replace(old.decode().encode('utf-16-le'),
                                   new.decode().encode('utf-16-le'))
open(path, 'wb').write(raw)
print('activity ->', new.decode())

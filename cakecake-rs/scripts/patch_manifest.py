"""把二进制 AndroidManifest.xml 里的 hasCode=false 改成 true。

cargo-apk 给纯 Rust 应用写的是 hasCode="false"，但我们现在要带 classes.dex
（返回键的 Java 壳），否则 Android 根本不会加载 dex，Activity 会 ClassNotFound。

做法：解析字符串池拿到 "hasCode" 的下标，再在属性记录里把同名属性的
布尔值从 0 改成 0xFFFFFFFF（等长改写，不动任何偏移）。
"""
import struct
import sys

path = sys.argv[1]
data = bytearray(open(path, 'rb').read())

# --- 1) 找字符串池，拿 hasCode 的下标 ---
assert struct.unpack_from('<H', data, 0)[0] == 0x0003, 'not a binary xml'
pool_off = 8
tag, header_size, size, str_count, style_count, flags, strings_start, styles_start = \
    struct.unpack_from('<HHIIIIII', data, pool_off)
assert tag == 0x0001, 'no string pool'
utf8 = bool(flags & 0x100)
offsets = [struct.unpack_from('<I', data, pool_off + header_size + i * 4)[0] for i in range(str_count)]


def read_str(i):
    start = pool_off + strings_start + offsets[i]
    if utf8:
        # u16len, u8len, bytes
        n = data[start]
        if n & 0x80:
            n = ((n & 0x7F) << 8) | data[start + 1]
            start += 2
        else:
            start += 1
        return data[start:start + n].decode('utf-8', 'ignore')
    n = struct.unpack_from('<H', data, start)[0]
    start += 2
    if n & 0x8000:
        n = ((n & 0x7FFF) << 16) | struct.unpack_from('<H', data, start)[0]
        start += 2
    return data[start:start + n * 2].decode('utf-16-le', 'ignore')


idx = None
for i in range(str_count):
    if read_str(i) == 'hasCode':
        idx = i
        break
if idx is None:
    print('hasCode not found')
    sys.exit(1)

# --- 2) 扫属性记录：name 下标 == idx 且 dataType == 0x12(boolean) ---
patched = 0
pos = 0
name_le = struct.pack('<I', idx)
while True:
    pos = data.find(name_le, pos)
    if pos < 0:
        break
    # 属性记录布局: ns(4) name(4) rawValue(4) size(2) res0(1) dataType(1) data(4)
    cand = pos - 4
    if cand >= 0:
        size2, res0, dtype = struct.unpack_from('<HBB', data, cand + 12)
        if size2 == 8 and res0 == 0 and dtype == 0x12:
            if struct.unpack_from('<I', data, cand + 16)[0] == 0:
                struct.pack_into('<I', data, cand + 16, 0xFFFFFFFF)
                patched += 1
    pos += 4

open(path, 'wb').write(data)
print(f'hasCode -> true ({patched} attr patched, string idx {idx})')

"""
生成 uni-app 移动端占位图标
- 4 个 TabBar 图标（灰色未选中）
- 4 个 TabBar 选中态（粉色）
- 1 个发布器中央浮动按钮（粉色圆）
- 1 个头像占位
- 3 个视频演示封面
"""
from PIL import Image, ImageDraw
import os

OUT = r'C:\Users\Administrator\Desktop\cakecake-project\cakecake-vue\cakecake-app\src\static'

# TabBar 颜色
GRAY = (153, 153, 153, 255)
PINK = (251, 114, 153, 255)
WHITE = (255, 255, 255, 255)


def make_tabbar(name: str, color: tuple, draw_func):
    """96x96 透明背景的 tabbar 图标"""
    img = Image.new('RGBA', (96, 96), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    draw_func(d, color)
    img.save(os.path.join(OUT, 'tabbar', f'{name}.png'))


# 首页 - 房子
def home(d, c):
    d.polygon([(48, 18), (18, 44), (18, 80), (78, 80), (78, 44)], outline=c, width=6)
    d.rectangle([(40, 56), (56, 80)], outline=c, width=4)

# 关注 - 心形
def follow(d, c):
    d.ellipse([(20, 24), (44, 48)], outline=c, width=6)
    d.ellipse([(52, 24), (76, 48)], outline=c, width=6)
    d.polygon([(48, 56), (18, 36), (78, 36)], outline=c, width=0) if False else None

# 会员购 - 购物袋
def mall(d, c):
    d.arc([(20, 24), (76, 80)], 0, 180, fill=c, width=6)
    d.line([(28, 30), (32, 50)], fill=c, width=6)
    d.line([(68, 30), (64, 50)], fill=c, width=6)
    d.rectangle([(20, 40), (76, 80)], outline=c, width=6)

# 我的 - 人头
def mine(d, c):
    d.ellipse([(36, 16), (60, 40)], outline=c, width=6)
    d.arc([(18, 48), (78, 96)], 0, 180, fill=c, width=6)

make_tabbar('home',        GRAY, home)
make_tabbar('home_sel',    PINK, home)
make_tabbar('follow',      GRAY, follow)
make_tabbar('follow_sel',  PINK, follow)
make_tabbar('mall',        GRAY, mall)
make_tabbar('mall_sel',    PINK, mall)
make_tabbar('mine',        GRAY, mine)
make_tabbar('mine_sel',    PINK, mine)

# 发布器中央浮动按钮（粉色圆 + 白色 +）
img = Image.new('RGBA', (140, 140), (0, 0, 0, 0))
d = ImageDraw.Draw(img)
d.ellipse([(0, 0), (140, 140)], fill=PINK)
d.line([(50, 70), (90, 70)], fill=WHITE, width=8)
d.line([(70, 50), (70, 90)], fill=WHITE, width=8)
img.save(os.path.join(OUT, 'tabbar', 'publish_center.png'))

# 头像占位
img = Image.new('RGBA', (200, 200), (220, 220, 220, 255))
d = ImageDraw.Draw(img)
d.ellipse([(60, 40), (140, 120)], fill=(180, 180, 180, 255))
d.arc([(20, 100), (180, 220)], 0, 180, fill=(180, 180, 180, 255))
img.save(os.path.join(OUT, 'avatar', 'default.png'))

# 占位视频封面（3 种色调）
for i, color in enumerate([(200, 180, 220), (180, 220, 200), (220, 200, 180)]):
    img = Image.new('RGB', (480, 300), color)
    d = ImageDraw.Draw(img)
    # 画一个播放按钮三角形
    cx, cy = 240, 150
    d.polygon([(cx - 30, cy - 50), (cx - 30, cy + 50), (cx + 50, cy)], fill=(255, 255, 255))
    img.save(os.path.join(OUT, 'demo', f'video{i+1}.jpg'))

# 占位 PNG（fallback）
img = Image.new('RGB', (480, 300), (240, 240, 240))
d = ImageDraw.Draw(img)
d.rectangle([(0, 0), (479, 299)], outline=(200, 200, 200), width=2)
img.save(os.path.join(OUT, 'placeholder.png'))

# logo
img = Image.new('RGBA', (200, 200), (251, 114, 153, 255))
d = ImageDraw.Draw(img)
d.ellipse([(40, 40), (160, 160)], fill=WHITE)
img.save(os.path.join(OUT, 'logo.png'))

# 商城 promo 占位
for i, color in enumerate([(255, 228, 236), (225, 245, 238), (230, 241, 251), (250, 238, 218)]):
    img = Image.new('RGB', (300, 300), color)
    d = ImageDraw.Draw(img)
    d.ellipse([(100, 60), (200, 160)], fill=(255, 255, 255))
    d.ellipse([(80, 150), (180, 250)], fill=(255, 255, 255))
    os.makedirs(os.path.join(OUT, 'mall'), exist_ok=True)
    img.save(os.path.join(OUT, 'mall', f'promo{i+1}.png'))

# 商城 flashsale 占位
for i in range(1, 3):
    img = Image.new('RGB', (300, 300), (220, 230, 240) if i == 1 else (240, 220, 240))
    d = ImageDraw.Draw(img)
    d.rectangle([(80, 50), (220, 250)], fill=(180, 180, 220))
    os.makedirs(os.path.join(OUT, 'mall'), exist_ok=True)
    img.save(os.path.join(OUT, 'mall', f'flash{i}.jpg'))

print('✅ All assets generated')
print('Files:')
for root, dirs, files in os.walk(OUT):
    for f in sorted(files):
        path = os.path.join(root, f)
        rel = os.path.relpath(path, OUT)
        print(f'  {rel} ({os.path.getsize(path)} bytes)')
//! 视频分区（和 Web 端 `constants/videoZones.js` 完全一致）
//!
//! 后端 `videos.zone` 存的是「父-子」形式（如 `生活-日常`），
//! `/api/v1/zones/:zone/recommendation` 同时接受父分区名和完整的父子分区名。

pub struct ZoneCat {
    pub name: &'static str,
    pub items: &'static [&'static str],
}

pub const ZONES: &[ZoneCat] = &[
    ZoneCat {
        name: "动画",
        items: &["MAD·AMV", "MMD·3D", "短片·手书·配音", "综合"],
    },
    ZoneCat {
        name: "番剧",
        items: &["连载动画", "完结动画", "资讯", "官方延伸", "新番时间表", "番剧索引"],
    },
    ZoneCat {
        name: "国创",
        items: &[
            "国产动画",
            "国产原创相关",
            "布袋戏",
            "资讯",
            "新番时间表",
            "国产动画索引",
        ],
    },
    ZoneCat {
        name: "音乐",
        items: &[
            "原创音乐",
            "翻唱",
            "VOCALOID·UTAU",
            "演奏",
            "三次元音乐",
            "OP/ED/OST",
            "音乐选集",
        ],
    },
    ZoneCat {
        name: "舞蹈",
        items: &["宅舞", "三次元舞蹈", "舞蹈教程"],
    },
    ZoneCat {
        name: "游戏",
        items: &[
            "单机游戏",
            "电子竞技",
            "手机游戏",
            "网络游戏",
            "桌游棋牌",
            "GMV",
            "音游",
            "Mugen",
        ],
    },
    ZoneCat {
        name: "科技",
        items: &[
            "趣味科普人文",
            "野生技术协会",
            "演讲·公开课",
            "星海",
            "数码",
            "机械",
            "汽车",
        ],
    },
    ZoneCat {
        name: "生活",
        items: &[
            "搞笑", "日常", "美食圈", "动物圈", "手工", "绘画", "ASMR", "运动", "其他",
        ],
    },
    ZoneCat {
        name: "鬼畜",
        items: &["鬼畜调教", "音MAD", "人力VOCALOID", "教程演示"],
    },
    ZoneCat {
        name: "时尚",
        items: &["美妆", "服饰", "健身", "资讯"],
    },
    ZoneCat {
        name: "广告",
        items: &[],
    },
    ZoneCat {
        name: "娱乐",
        items: &["综艺", "明星", "Korea相关"],
    },
    ZoneCat {
        name: "影视",
        items: &["影视杂谈", "影视剪辑", "短片", "预告·资讯", "特摄"],
    },
    ZoneCat {
        name: "放映厅",
        items: &["纪录片", "电影", "电视剧"],
    },
];

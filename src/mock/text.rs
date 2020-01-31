use rand::{thread_rng, Rng};
use crate::mock;

const CHINESE_CHARS: [char; 500] = ['的', '一', '是', '在', '不', '了', '有', '和', '人', '这', '中', '大', '为', '上', '个', '国', '我', '以', '要', '他', '时', '来', '用', '们', '生', '到', '作', '地', '于', '出', '就', '分', '对', '成', '会', '可', '主', '发', '年', '动', '同', '工', '也', '能', '下', '过', '子', '说', '产', '种', '面', '而', '方', '后', '多', '定', '行', '学', '法', '所', '民', '得', '经', '十', '三', '之', '进', '着', '等', '部', '度', '家', '电', '力', '里', '如', '水', '化', '高', '自', '二', '理', '起', '小', '物', '现', '实', '加', '量', '都', '两', '体', '制', '机', '当', '使', '点', '从', '业', '本', '去', '把', '性', '好', '应', '开', '它', '合', '还', '因', '由', '其', '些', '然', '前', '外', '天', '政', '四', '日', '那', '社', '义', '事', '平', '形', '相', '全', '表', '间', '样', '与', '关', '各', '重', '新', '线', '内', '数', '正', '心', '反', '你', '明', '看', '原', '又', '么', '利', '比', '或', '但', '质', '气', '第', '向', '道', '命', '此', '变', '条', '只', '没', '结', '解', '问', '意', '建', '月', '公', '无', '系', '军', '很', '情', '者', '最', '立', '代', '想', '已', '通', '并', '提', '直', '题', '党', '程', '展', '五', '果', '料', '象', '员', '革', '位', '入', '常', '文', '总', '次', '品', '式', '活', '设', '及', '管', '特', '件', '长', '求', '老', '头', '基', '资', '边', '流', '路', '级', '少', '图', '山', '统', '接', '知', '较', '将', '组', '见', '计', '别', '她', '手', '角', '期', '根', '论', '运', '农', '指', '几', '九', '区', '强', '放', '决', '西', '被', '干', '做', '必', '战', '先', '回', '则', '任', '取', '据', '处', '队', '南', '给', '色', '光', '门', '即', '保', '治', '北', '造', '百', '规', '热', '领', '七', '海', '口', '东', '导', '器', '压', '志', '世', '金', '增', '争', '济', '阶', '油', '思', '术', '极', '交', '受', '联', '什', '认', '六', '共', '权', '收', '证', '改', '清', '己', '美', '再', '采', '转', '更', '单', '风', '切', '打', '白', '教', '速', '花', '带', '安', '场', '身', '车', '例', '真', '务', '具', '万', '每', '目', '至', '达', '走', '积', '示', '议', '声', '报', '斗', '完', '类', '八', '离', '华', '名', '确', '才', '科', '张', '信', '马', '节', '话', '米', '整', '空', '元', '况', '今', '集', '温', '传', '土', '许', '步', '群', '广', '石', '记', '需', '段', '研', '界', '拉', '林', '律', '叫', '且', '究', '观', '越', '织', '装', '影', '算', '低', '持', '音', '众', '书', '布', '复', '容', '儿', '须', '际', '商', '非', '验', '连', '断', '深', '难', '近', '矿', '千', '周', '委', '素', '技', '备', '半', '办', '青', '省', '列', '习', '响', '约', '支', '般', '史', '感', '劳', '便', '团', '往', '酸', '历', '市', '克', '何', '除', '消', '构', '府', '称', '太', '准', '精', '值', '号', '率', '族', '维', '划', '选', '标', '写', '存', '候', '毛', '亲', '快', '效', '斯', '院', '查', '江', '型', '眼', '王', '按', '格', '养', '易', '置', '派', '层', '片', '始', '却', '专', '状', '育', '厂', '京', '识', '适', '属', '圆', '包', '火', '住', '调', '满', '县', '局', '照', '参', '红', '细', '引', '听', '该', '铁', '价', '严', '龙', '飞'];
const CHINESE_PUNCTUATION: [char; 9] = ['，', '。', '，', '！', '，', '？', '，', '；', '，'];
const CHINESE_PUNCTUATION2: [char; 4] = [':', '、', '-', '"'];
const CHINESE_PUNCTUATION3: [char; 4] = ['！', '？', ' ', ' '];

const EN_PUNCTUATION: [char; 9] = [',', '.', ',', '!', ',', '?', ',', ';', ','];
// 句子结尾符号
const EN_PUNCTUATION2: [char; 4] = [':', '.', '-', '"'];
// 语句中间的符号
const EN_PUNCTUATION3: [char; 4] = ['！', '？', ' ', ' ']; // 标题结尾符号

/// 生成随机中文段落
/// length 表示有几个句子
pub fn cparagraph(mut length: u64, mut min_length: u64, mut max_length: u64, content_type:&str) -> String {
    let mut s = String::new();
    let mut rng = thread_rng();

    if min_length == 0 {
        min_length = 300;
    }
    if max_length == 0 {
        max_length = 1600;
    }

    if length == 0 {
        // 默认有5到10个句子
        length = rng.gen_range(min_length, max_length);
    }

    let length = (length * 3) as usize;
    loop {
        let s1 = csummary(0, 0, 0);
        if (s.len() + s1.len()) >= length {
            // 整个title长度不能超过length长度
            break;
        }
        if content_type == "html" {
            s.push_str("<p>");
        }
        s.push_str(&s1);
        if content_type == "html" {
            s.push_str("</p>\n");
        } else if content_type == "markdown" {
            s.push_str("\n\n");
        }
    }

    if s.ends_with("，") {
        let x = s.trim_end_matches("，");
        return format!("{}。", x);
    }
    s
}

/// 随机生成英文段落
pub fn paragraph(mut length: u64, mut min_length: u64, mut max_length: u64, content_type:&str) -> String {
    let mut s = String::new();
    let mut rng = thread_rng();

    if min_length == 0 {
        min_length = 300;
    }
    if max_length == 0 {
        max_length = 1600;
    }

    if length == 0 {
        // 默认有5到10个句子
        length = rng.gen_range(min_length, max_length);
    }

    let length = length as usize;
    loop {
        let s1 = summary(0, 0, 0);
        if (s.len() + s1.len()) >= length {
            // 整个title长度不能超过length长度
            break;
        }
        if content_type == "html" {
            s.push_str("<p>");
        }
        s.push_str(&s1);
        if content_type == "html" {
            s.push_str("</p>\n");
        } else if content_type == "markdown" {
            s.push_str("\n\n");
        }
    }

    s
}


/// 生成随机中文小段落
pub fn csummary(mut length: u64, mut min_length: u64, mut max_length: u64) -> String {
    let mut s = String::new();
    let mut rng = thread_rng();
    if min_length == 0 {
        min_length = 60;
    }
    if max_length == 0 {
        max_length = 250;
    }

    if length == 0 {
        length = rng.gen_range(min_length, max_length);
    }

    let length = (length * 3) as usize;
    loop {
        let s1 = csentence(0, 0, 0);
        if (s.len() + s1.len()) >= length {
            // 整个title长度不能超过length长度
            break;
        }
        s.push_str(&s1);
    }

    if s.ends_with("，") {
        let x = s.trim_end_matches("，");
        return format!("{}。", x);
    }
    s
}


/// 生成随机英文小段落
pub fn summary(mut length: u64, mut min_length: u64, mut max_length: u64) -> String {
    let mut s = String::new();
    let mut rng = thread_rng();
    if min_length == 0 {
        min_length = 120;
    }
    if max_length == 0 {
        max_length = 300;
    }

    if length == 0 {
        length = rng.gen_range(min_length, max_length);
    }
    let length = length as usize;
    loop {
        let s1 = sentence(0, 0, 0);
        if (s.len() + s1.len()) >= length {
            // 整个title长度不能超过length长度
            break;
        }
        s.push_str(&s1);
    }

    if s.ends_with(", ") {
        let x = s.trim_end_matches(", ");
        return format!("{}.", x);
    }
    s
}


/// 生成随机中文句子
pub fn csentence(mut length: u64, mut min_length: u64, mut max_length: u64) -> String {
    let mut s = String::new();
    let mut rng = thread_rng();
    if min_length == 0 {
        min_length = 5;
    }
    if max_length == 0 {
        max_length = 50;
    }

    if length == 0 {
        length = rng.gen_range(min_length, max_length);
    }

    let length = (length * 3) as usize;
    loop {
        let s1 = cword(0, 0, 0);
        if (s.len() + s1.len()) >= length {
            // 整个title长度不能超过length长度
            break;
        }
        s.push_str(&s1);
    }

    let s2 = cpunctuation(0);
    format!("{}{}", s, s2)
}


/// 生成随机英文句子
pub fn sentence(mut length: u64, mut min_length: u64, mut max_length: u64) -> String {
    let mut s = String::new();
    let mut rng = thread_rng();
    if min_length == 0 {
        min_length = 20;
    }
    if max_length == 0 {
        max_length = 90;
    }

    if length == 0 {
        length = rng.gen_range(min_length, max_length);
    }

    let length = length as usize;
    let c = mock::basic::alphabet();
    s = c.to_uppercase().to_string();
    loop {
        let s1 = word(0, 0, 0);
        if (s.len() + s1.len()) >= length {
            // 整个title长度不能超过length长度
            break;
        }
        s.push_str(&s1);
        s.push_str(" ");
    }

    let s2 = punctuation(0);
    format!("{}{} ", s.trim(), s2)
}


/// 生成随机中文标题
pub fn ctitle(mut length: u64, mut min_length: u64, mut max_length: u64) -> String {
    let mut s = String::new();
    let mut rng = thread_rng();
    if min_length == 0 {
        min_length = 5;
    }
    if max_length == 0 {
        max_length = 50;
    }

    if length == 0 {
        length = rng.gen_range(min_length, max_length);
    }

    let length = (length * 3) as usize;
    loop {
        let s1 = cword(0, 0, 0);
        if (s.len() + s1.len()) >= length {
            // 整个title长度不能超过length长度
            break;
        }
        s.push_str(&s1);
    }

    let s2 = cpunctuation(3);
    if &s2 == " " {
        return s;
    }
    format!("{}{}", s, s2)
}

/// 生成随机英文标题
pub fn title(mut length: u64, mut min_length: u64, mut max_length: u64) -> String {
    let mut s = String::new();
    let mut rng = thread_rng();
    if min_length == 0 {
        min_length = 20;
    }
    if max_length == 0 {
        max_length = 90;
    }

    if length == 0 {
        length = rng.gen_range(min_length, max_length);
    }

    let length = length as usize;
    let c = mock::basic::alphabet();
    s = c.to_uppercase().to_string();
    loop {
        let s1 = word(0, 0, 0);
        if (s.len() + s1.len()) >= length {
            // 整个title长度不能超过length长度
            break;
        }
        s.push_str(&s1);
        s.push_str(" ");
    }

    let s2 = punctuation(3);
    if &s2 == " " {
        return s.trim().to_string();
    }
    format!("{}{}", s.trim(), s2)
}

/// 生成中文随机结尾标点符号
pub fn cpunctuation(index: usize) -> String {
    let mut s = String::new();
    let mut rng = thread_rng();

    let mut a1;
    if index == 3 {
        let n: usize = rng.gen_range(0, CHINESE_PUNCTUATION3.len());
        a1 = &CHINESE_PUNCTUATION3[n];
    } else if index == 2 {
        let n: usize = rng.gen_range(0, CHINESE_PUNCTUATION2.len());
        a1 = &CHINESE_PUNCTUATION2[n];
    } else {
        let n: usize = rng.gen_range(0, CHINESE_PUNCTUATION.len());
        a1 = &CHINESE_PUNCTUATION[n];
    }
    s.push(*a1);
    s
}

/// 生成英文随机结尾标点符号
pub fn punctuation(index: usize) -> String {
    let mut s = String::new();
    let mut rng = thread_rng();

    let mut a1;
    if index == 3 {
        let n: usize = rng.gen_range(0, EN_PUNCTUATION3.len());
        a1 = &EN_PUNCTUATION3[n];
    } else if index == 2 {
        let n: usize = rng.gen_range(0, EN_PUNCTUATION2.len());
        a1 = &EN_PUNCTUATION2[n];
    } else {
        let n: usize = rng.gen_range(0, EN_PUNCTUATION.len());
        a1 = &EN_PUNCTUATION[n];
    }
    s.push(*a1);
    s
}


/// 生成随机英文单词
pub fn word(mut length: usize, mut min_length: u64, mut max_length: u64) -> String {
    let mut rng = thread_rng();
    let mut s = String::new();

    if min_length == 0 {
        min_length = 3;
    }
    if max_length == 0 {
        max_length = 10;
    }
    if length == 0 {
        length = rng.gen_range(min_length as usize, max_length as usize);
    }

    while length > 0 {
        s.push(mock::basic::alphabet());
        length -= 1;
    }
    s
}


/// 生成随机中文单词
pub fn cword(mut length: usize, mut min_length: u64, mut max_length: u64) -> String {
    let mut s = String::new();
    let mut rng = thread_rng();

    if min_length == 0 {
        min_length = 1;
    }
    if max_length == 0 {
        max_length = 4;
    }
    if length == 0 {
        length = rng.gen_range(min_length as usize, max_length as usize);
    }

    while length > 0 {
        let n: usize = rng.gen_range(0, 500);
        let a1 = &CHINESE_CHARS[n];
        s.push(*a1);
        length -= 1;
    }
    s
}

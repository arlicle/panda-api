use rand::{thread_rng, Rng};
use crate::mock;

const CHINESE_CHARS: [char; 500] = ['的', '一', '是', '在', '不', '了', '有', '和', '人', '这', '中', '大', '为', '上', '个', '国', '我', '以', '要', '他', '时', '来', '用', '们', '生', '到', '作', '地', '于', '出', '就', '分', '对', '成', '会', '可', '主', '发', '年', '动', '同', '工', '也', '能', '下', '过', '子', '说', '产', '种', '面', '而', '方', '后', '多', '定', '行', '学', '法', '所', '民', '得', '经', '十', '三', '之', '进', '着', '等', '部', '度', '家', '电', '力', '里', '如', '水', '化', '高', '自', '二', '理', '起', '小', '物', '现', '实', '加', '量', '都', '两', '体', '制', '机', '当', '使', '点', '从', '业', '本', '去', '把', '性', '好', '应', '开', '它', '合', '还', '因', '由', '其', '些', '然', '前', '外', '天', '政', '四', '日', '那', '社', '义', '事', '平', '形', '相', '全', '表', '间', '样', '与', '关', '各', '重', '新', '线', '内', '数', '正', '心', '反', '你', '明', '看', '原', '又', '么', '利', '比', '或', '但', '质', '气', '第', '向', '道', '命', '此', '变', '条', '只', '没', '结', '解', '问', '意', '建', '月', '公', '无', '系', '军', '很', '情', '者', '最', '立', '代', '想', '已', '通', '并', '提', '直', '题', '党', '程', '展', '五', '果', '料', '象', '员', '革', '位', '入', '常', '文', '总', '次', '品', '式', '活', '设', '及', '管', '特', '件', '长', '求', '老', '头', '基', '资', '边', '流', '路', '级', '少', '图', '山', '统', '接', '知', '较', '将', '组', '见', '计', '别', '她', '手', '角', '期', '根', '论', '运', '农', '指', '几', '九', '区', '强', '放', '决', '西', '被', '干', '做', '必', '战', '先', '回', '则', '任', '取', '据', '处', '队', '南', '给', '色', '光', '门', '即', '保', '治', '北', '造', '百', '规', '热', '领', '七', '海', '口', '东', '导', '器', '压', '志', '世', '金', '增', '争', '济', '阶', '油', '思', '术', '极', '交', '受', '联', '什', '认', '六', '共', '权', '收', '证', '改', '清', '己', '美', '再', '采', '转', '更', '单', '风', '切', '打', '白', '教', '速', '花', '带', '安', '场', '身', '车', '例', '真', '务', '具', '万', '每', '目', '至', '达', '走', '积', '示', '议', '声', '报', '斗', '完', '类', '八', '离', '华', '名', '确', '才', '科', '张', '信', '马', '节', '话', '米', '整', '空', '元', '况', '今', '集', '温', '传', '土', '许', '步', '群', '广', '石', '记', '需', '段', '研', '界', '拉', '林', '律', '叫', '且', '究', '观', '越', '织', '装', '影', '算', '低', '持', '音', '众', '书', '布', '复', '容', '儿', '须', '际', '商', '非', '验', '连', '断', '深', '难', '近', '矿', '千', '周', '委', '素', '技', '备', '半', '办', '青', '省', '列', '习', '响', '约', '支', '般', '史', '感', '劳', '便', '团', '往', '酸', '历', '市', '克', '何', '除', '消', '构', '府', '称', '太', '准', '精', '值', '号', '率', '族', '维', '划', '选', '标', '写', '存', '候', '毛', '亲', '快', '效', '斯', '院', '查', '江', '型', '眼', '王', '按', '格', '养', '易', '置', '派', '层', '片', '始', '却', '专', '状', '育', '厂', '京', '识', '适', '属', '圆', '包', '火', '住', '调', '满', '县', '局', '照', '参', '红', '细', '引', '听', '该', '铁', '价', '严', '龙', '飞'];
const CHINESE_PUNCTUATION: [char; 5] = ['。', '，', '！', '？', '；'];
const CHINESE_PUNCTUATION2: [char; 4] = [':', '、', '-', '"'];
const CHINESE_PUNCTUATION3: [char; 4] = ['！', '？', ' ', ' '];


///// 生成随机中文句子
//pub fn csentence(mut length: usize) -> String {
//    let mut s = String::new();
//    let mut rng = thread_rng();
//    let n: usize = rng.gen_range(0, 500);
//    if n + length > 500 {
//        let a1 = &CHINESE_CHARS[n..500];
//        let a1_str: String = a1.into_iter().collect();
//        s.push_str(&a1_str);
//        let a2 = &CHINESE_CHARS[0..(length - (500 - n))];
//        let a2_str: String = a2.into_iter().collect();
//        s.push_str(&a2_str);
//    } else {
//        let a = &CHINESE_CHARS[n..=n + length];
//        let a_str: String = a.into_iter().collect();
//        s.push_str(&a_str);
//    }
//    s
//}


/// 生成随机中文段落
/// length 表示有几个句子
pub fn cparagraph(mut length: usize) -> String {
    let mut s = String::new();
    let mut rng = thread_rng();
    if length == 0 {
        // 默认有5到10个句子
        length = rng.gen_range(5, 10);
    }

    while length > 0 {
        let s1 = csentence(0);
        s.push_str(&s1);
        length -= 1;
    }
    if s.ends_with("，") {
        let x = s.trim_end_matches("，");
        return format!("{}。", x);
    }
    s
}


/// 生成随机中文句子
pub fn csentence(mut length: usize) -> String {
    let mut s = String::new();
    let mut rng = thread_rng();
    if length == 0 {
        length = rng.gen_range(12, 20);
    }
    let s1 = cword(length);
    let s2 = cpunctuation(0);
    format!("{}{}", s1, s2)
}


/// 生成随机中文标题
pub fn ctitle(mut length: usize) -> String {
    let mut s = String::new();
    let mut rng = thread_rng();
    if length == 0 {
        length = rng.gen_range(5, 20);
    }
    let s1 = cword(length);
    let s2 = cpunctuation(3);
    if &s2 == " " {
        return s1;
    }
    format!("{}{}", s1, s2)
}

/// 生成随机结尾标点符号
pub fn cpunctuation(index: usize) -> String {
    let mut s = String::new();
    let mut rng = thread_rng();

    let mut a1;
    if index == 3 {
        let n: usize = rng.gen_range(0, 4);
        a1 = &CHINESE_PUNCTUATION3[n];
    } else if index == 2 {
        let n: usize = rng.gen_range(0, 4);
        a1 = &CHINESE_PUNCTUATION2[n];
    } else {
        let n: usize = rng.gen_range(0, 5);
        a1 = &CHINESE_PUNCTUATION[n];
    }
    s.push(*a1);


    s
}


/// 生成随机英文单词
pub fn word() -> String {
    let mut rng = thread_rng();
    let mut n = rng.gen_range(3, 10);
    let mut s = String::new();
    while n > 0 {
        s.push(mock::basic::char());
    }

    s
}



/// 生成随机中文单词
pub fn cword(mut length: usize) -> String {
    let mut s = String::new();
    let mut rng = thread_rng();

    if length == 0 {
        length = rng.gen_range(1, 4);
    }

    while length > 0 {
        let n: usize = rng.gen_range(0, 500);
        let a1 = &CHINESE_CHARS[n];
        s.push(*a1);
        length -= 1;
    }
    s
}

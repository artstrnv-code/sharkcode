#[derive(Component, Clone, Copy)]
enum Action { Select(usize), UseEntry, ShowDebug, ToggleCodex, CodexPage(CodexPage), Run, NewRun, QuizAnswer(usize), ProjectTab(usize), ToggleInventory, CloseCompletion }
#[derive(Component)] struct StageText;
#[derive(Component)] struct MessageText;
#[derive(Component)] struct ReportScroll;
#[derive(Component)] struct StatusText;
#[derive(Component)] struct CodeEditor;
#[derive(Component)] struct EditorPanel;
#[derive(Component)] struct RunPanel;
#[derive(Component)] struct DebugPanel;
#[derive(Component)] struct EntryPanel;
#[derive(Component)] struct ConsolePanel;
#[derive(Component)] struct ConsoleLabel;
#[derive(Component)] struct InventoryPanel;
#[derive(Component)] struct InventoryText;
#[derive(Component)] struct CompletionPanel;
#[derive(Component)] struct CompletionText;
#[derive(Resource)]
struct ProjectEditor { active: usize, files: [String; 4], last_selected: usize }
impl ProjectEditor {
    fn reset(&mut self) {
        *self = Self::default();
    }
}
impl Default for ProjectEditor {
    fn default() -> Self {
        Self { active: 3, last_selected: usize::MAX, files: [
            "[build-system]\nrequires = [\"hatchling\"]\nbuild-backend = \"hatchling.build\"\n\n[project]\nname = \"name_lib\"\nversion = \"0.0.0\"\nreadme = \"README.md\"\n".into(),
            "# name_lib\n".into(),
            "from .core import dragon, fire, python\n".into(),
            "def dragon(a):\n    pass\n\ndef fire(a):\n    pass\n\ndef python(a):\n    pass\n".into(),
        ], }
    }
}
#[derive(Component)] struct ProjectTabs;
#[derive(Resource,Default)] struct DevInput { line: String }
#[derive(Resource,Default)] struct TextReveal { stage: String, message: String, stage_visible: usize, message_visible: usize, elapsed: f32 }

#[derive(Component)] struct CodexPanel;
#[derive(Component)] struct StagePanel;
#[derive(Component)] struct CodexText;
#[derive(Component)] struct CodexEntry(CodexPage);
#[derive(Component)] struct CodexScroll;
#[derive(Component)] struct CodexListScroll;
#[derive(Component)] struct MapScroll;
#[derive(Component)] struct QuizPanel;
#[derive(Component)] struct QuizButtonText(usize);
#[derive(Component)] struct QuizAnswerButton(usize);

// Простой редактор для Bevy 0.18: EditableText появился только в 0.19.
// Курсор хранится как байтовая позиция на границе UTF-8.
#[derive(Resource, Default)]
struct CodeBuffer {
    source: String, cursor: usize, anchor: Option<usize>, preferred_column: Option<usize>,
    undo: Vec<(String, usize)>, redo: Vec<(String, usize)>, dragging: bool,
}
impl CodeBuffer {
    // Переход в другой нод = новый документ; Ctrl+Z не должен возвращать
    // содержимое предыдущего испытания.
    fn reset(&mut self) {
        self.source.clear();
        self.cursor = 0;
        self.anchor = None;
        self.preferred_column = None;
        self.undo.clear();
        self.redo.clear();
        self.dragging = false;
    }
    fn save_history(&mut self) {
        self.undo.push((self.source.clone(), self.cursor));
        if self.undo.len() > 150 { self.undo.remove(0); }
        self.redo.clear();
    }
    fn selection(&self) -> Option<(usize,usize)> {
        self.anchor.filter(|a| *a != self.cursor).map(|a| (a.min(self.cursor),a.max(self.cursor)))
    }
    fn erase_selection(&mut self) -> bool {
        if let Some((a,b)) = self.selection() {
            self.source.replace_range(a..b, ""); self.cursor = a; self.anchor = None; true
        } else {false}
    }
    fn insert(&mut self, text: &str) {
        self.save_history(); self.erase_selection(); self.source.insert_str(self.cursor,text);
        self.cursor += text.len(); self.preferred_column = None;
    }
    // «Двойной щит»: автопарные круглые скобки. Один шаг истории = одна пара.
    // При выделении оборачиваем фрагмент, а не удаляем его.
    fn insert_open_paren(&mut self) {
        self.save_history();
        if let Some((start, end)) = self.selection() {
            self.source.insert(end, ')');
            self.source.insert(start, '(');
            self.cursor = end + 1; // перед закрывающей скобкой
            self.anchor = None;
        } else {
            self.source.insert_str(self.cursor, "()");
            self.cursor += 1;
        }
        self.preferred_column = None;
    }
    fn insert_close_paren(&mut self) {
        // Не дублируем уже автоматически поставленную закрывающую скобку.
        if self.selection().is_none() && self.source[self.cursor..].starts_with(')') {
            self.move_right(false);
        } else {
            self.insert(")");
        }
    }
    fn backspace(&mut self) {
        if self.selection().is_some() {self.save_history(); self.erase_selection(); return;}
        if self.cursor > 0 {self.save_history(); self.move_left(false);
            let next = self.source[self.cursor..].chars().next().unwrap().len_utf8();
            self.source.replace_range(self.cursor..self.cursor+next, "");}
    }
    fn delete(&mut self) {
        if self.selection().is_some() {self.save_history(); self.erase_selection(); return;}
        if self.cursor < self.source.len() { self.save_history();
            let next = self.source[self.cursor..].chars().next().unwrap().len_utf8();
            self.source.replace_range(self.cursor..self.cursor+next, ""); }
    }
    fn move_to(&mut self, pos:usize, select:bool) {
        if select { if self.anchor.is_none() {self.anchor = Some(self.cursor);} }
        else {self.anchor = None;}
        self.cursor = pos.min(self.source.len());
    }
    fn move_left(&mut self, select:bool) {
        let pos = self.source[..self.cursor].char_indices().last().map(|(i,_)| i).unwrap_or(0);
        self.move_to(pos,select); self.preferred_column = None;
    }
    fn move_right(&mut self, select:bool) {
        let pos = self.cursor + self.source[self.cursor..].chars().next().map_or(0,char::len_utf8);
        self.move_to(pos,select); self.preferred_column = None;
    }
    fn cursor_position(&self) -> (usize,usize) {
        let prefix=&self.source[..self.cursor];
        (prefix.bytes().filter(|b|*b==b'\n').count(),prefix.rsplit('\n').next().unwrap_or("").chars().count())
    }
    fn line_start(&self) -> usize {self.source[..self.cursor].rfind('\n').map_or(0,|x|x+1)}
    fn line_end(&self) -> usize {self.source[self.cursor..].find('\n').map_or(self.source.len(),|x|self.cursor+x)}
    fn move_vertical(&mut self, up:bool, select:bool) {
        let (row,col)=self.cursor_position();
        let preferred=*self.preferred_column.get_or_insert(col);
        let target=if up {row.saturating_sub(1)} else {row+1};
        let pos=self.position_at(target,preferred);
        self.move_to(pos,select);
    }
    fn position_at(&self,row:usize,col:usize)->usize {
        let mut offset=0;
        for (index,line) in self.source.split('\n').enumerate(){
            if index==row {
                return offset+line.char_indices().nth(col).map(|(i,_)|i).unwrap_or(line.len());
            }
            offset+=line.len()+1;
        }
        self.source.len()
    }
    fn undo(&mut self, redo:bool){
        let (src,dst)=if redo {(&mut self.redo,&mut self.undo)} else {(&mut self.undo,&mut self.redo)};
        if let Some(previous)=src.pop(){dst.push((self.source.clone(),self.cursor));
            (self.source,self.cursor)=previous;self.anchor=None;}
    }
}
// Рисуем каретку в том же Text, что и сам код. Это устраняет
// рассинхронизацию координат отдельного абсолютного UI-элемента.
// Символ каретки добавляется только к отображению; в исходный код и
// валидацию он никогда не попадает.
fn display_code(buffer: &CodeBuffer, numbered: bool) -> String {
    let mut displayed = buffer.source.clone();
    displayed.insert(buffer.cursor, '│');
    if numbered {
        // Prefix only in the visible editor, never in submitted Python code.
        let width = displayed.lines().count().max(1).to_string().len().max(2);
        displayed = displayed.split('\n').enumerate().map(|(i,line)|
            format!("{:>width$} │ {}",i+1,line,width=width)).collect::<Vec<_>>().join("\n");
    }
    displayed
}

// Простой подсветчик ключевых слов для награды «Золотой идол».
// Обходит строки и комментарии, чтобы не красить совпадения внутри текста.
fn keyword_segments(text: &str) -> Vec<(String, bool)> {
    let mut result = Vec::new();
    let mut fragment = String::new();
    let mut chars = text.chars().peekable();
    let mut quote: Option<char> = None;
    let mut comment = false;
    let keywords = ["False", "None", "True", "and", "as", "assert", "async", "await",
        "break", "class", "continue", "def", "del", "elif", "else", "except",
        "finally", "for", "from", "global", "if", "import", "in", "is", "lambda",
        "nonlocal", "not", "or", "pass", "raise", "return", "try", "while",
        "with", "yield", "match", "case"];
    while let Some(ch) = chars.next() {
        if comment {
            fragment.push(ch);
            if ch == '\n' {comment=false;}
            continue;
        }
        if let Some(delimiter) = quote {
            fragment.push(ch);
            if ch == '\\' {
                if let Some(next) = chars.next() {fragment.push(next);}
            } else if ch == delimiter { quote = None; }
            continue;
        }
        if ch == '#' {comment=true;fragment.push(ch);continue;}
        if ch == '\'' || ch == '"' {quote=Some(ch);fragment.push(ch);continue;}
        if ch.is_ascii_alphabetic() || ch == '_' {
            let mut word = String::from(ch);
            while chars.peek().is_some_and(|c|c.is_ascii_alphanumeric()||*c=='_') {
                word.push(chars.next().unwrap());
            }
            let is_keyword=keywords.contains(&word.as_str());
            if is_keyword {
                if !fragment.is_empty() {result.push((std::mem::take(&mut fragment),false));}
                result.push((word,true));
            } else { fragment.push_str(&word); }
        } else {fragment.push(ch);}
    }
    if !fragment.is_empty() {result.push((fragment,false));}
    result
}

// Ввод осуществляется событиями KeyboardInput, а не недоступным в Bevy 0.18 EditableText.

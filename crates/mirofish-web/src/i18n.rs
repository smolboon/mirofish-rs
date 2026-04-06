//! Internationalization (i18n) support for the web frontend

use std::collections::HashMap;

/// Supported languages
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Language {
    English,
    Chinese,
}

impl Language {
    pub fn code(&self) -> &str {
        match self {
            Language::English => "en",
            Language::Chinese => "zh",
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Language::English => "English",
            Language::Chinese => "中文",
        }
    }
}

/// Translation store
pub struct I18n {
    current_language: Language,
    translations: HashMap<Language, HashMap<String, String>>,
}

impl I18n {
    pub fn new() -> Self {
        let mut translations = HashMap::new();

        // English translations
        let mut en = HashMap::new();
        en.insert("app_title".to_string(), "MiroFish - Social Simulation Platform".to_string());
        en.insert("home".to_string(), "Home".to_string());
        en.insert("create_project".to_string(), "Create Project".to_string());
        en.insert("project_name".to_string(), "Project Name".to_string());
        en.insert("simulation_requirement".to_string(), "Simulation Requirement".to_string());
        en.insert("next".to_string(), "Next".to_string());
        en.insert("previous".to_string(), "Previous".to_string());
        en.insert("start".to_string(), "Start".to_string());
        en.insert("stop".to_string(), "Stop".to_string());
        en.insert("pause".to_string(), "Pause".to_string());
        en.insert("resume".to_string(), "Resume".to_string());
        en.insert("step1_graph".to_string(), "Step 1: Build Knowledge Graph".to_string());
        en.insert("step2_env".to_string(), "Step 2: Environment Setup".to_string());
        en.insert("step3_simulation".to_string(), "Step 3: Run Simulation".to_string());
        en.insert("step4_report".to_string(), "Step 4: Generate Report".to_string());
        en.insert("step5_interaction".to_string(), "Step 5: Interact with Agents".to_string());
        en.insert("enable_twitter".to_string(), "Enable Twitter Simulation".to_string());
        en.insert("enable_reddit".to_string(), "Enable Reddit Simulation".to_string());
        en.insert("upload_files".to_string(), "Upload Files".to_string());
        en.insert("graph_visualization".to_string(), "Graph Visualization".to_string());
        en.insert("simulation_progress".to_string(), "Simulation Progress".to_string());
        en.insert("report".to_string(), "Report".to_string());
        en.insert("interview_agent".to_string(), "Interview Agent".to_string());
        en.insert("loading".to_string(), "Loading...".to_string());
        en.insert("error".to_string(), "Error".to_string());
        en.insert("success".to_string(), "Success".to_string());
        translations.insert(Language::English, en);

        // Chinese translations
        let mut zh = HashMap::new();
        zh.insert("app_title".to_string(), "MiroFish - 社交模拟平台".to_string());
        zh.insert("home".to_string(), "首页".to_string());
        zh.insert("create_project".to_string(), "创建项目".to_string());
        zh.insert("project_name".to_string(), "项目名称".to_string());
        zh.insert("simulation_requirement".to_string(), "模拟需求".to_string());
        zh.insert("next".to_string(), "下一步".to_string());
        zh.insert("previous".to_string(), "上一步".to_string());
        zh.insert("start".to_string(), "开始".to_string());
        zh.insert("stop".to_string(), "停止".to_string());
        zh.insert("pause".to_string(), "暂停".to_string());
        zh.insert("resume".to_string(), "继续".to_string());
        zh.insert("step1_graph".to_string(), "步骤1：构建知识图谱".to_string());
        zh.insert("step2_env".to_string(), "步骤2：环境设置".to_string());
        zh.insert("step3_simulation".to_string(), "步骤3：运行模拟".to_string());
        zh.insert("step4_report".to_string(), "步骤4：生成报告".to_string());
        zh.insert("step5_interaction".to_string(), "步骤5：与智能体交互".to_string());
        zh.insert("enable_twitter".to_string(), "启用Twitter模拟".to_string());
        zh.insert("enable_reddit".to_string(), "启用Reddit模拟".to_string());
        zh.insert("upload_files".to_string(), "上传文件".to_string());
        zh.insert("graph_visualization".to_string(), "图谱可视化".to_string());
        zh.insert("simulation_progress".to_string(), "模拟进度".to_string());
        zh.insert("report".to_string(), "报告".to_string());
        zh.insert("interview_agent".to_string(), "访谈智能体".to_string());
        zh.insert("loading".to_string(), "加载中...".to_string());
        zh.insert("error".to_string(), "错误".to_string());
        zh.insert("success".to_string(), "成功".to_string());
        translations.insert(Language::Chinese, zh);

        Self {
            current_language: Language::English,
            translations,
        }
    }

    /// Get translation for a key
    pub fn t(&self, key: &str) -> String {
        self.translations
            .get(&self.current_language)
            .and_then(|lang| lang.get(key))
            .cloned()
            .unwrap_or(key.to_string())
    }

    /// Set current language
    pub fn set_language(&mut self, language: Language) {
        self.current_language = language;
    }

    /// Get current language
    pub fn current_language(&self) -> &Language {
        &self.current_language
    }

    /// Get all available languages
    pub fn available_languages(&self) -> Vec<Language> {
        vec![Language::English, Language::Chinese]
    }
}

impl Default for I18n {
    fn default() -> Self {
        Self::new()
    }
}
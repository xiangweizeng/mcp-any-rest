<script setup lang="ts">
import { Layout } from "vuepress-theme-hope/client";
import { useRouteLocale } from "@vuepress/client";
import { computed } from "vue";
// @ts-ignore
import zhReadme from "../../about/README.md?raw";
// @ts-ignore
import enReadme from "../../en/about/README.md?raw";

const routeLocale = useRouteLocale();

const i18n = {
  "/": {
    header: {
      tagline: "打破孤岛，连接万物的通用协议层",
      quickStart: "快速开始"
    },
    about: {
      title: "关于 MCP-ANY-REST",
      desc: "MCP-ANY-REST 致力于构建打破数据孤岛的通用协议层。作为官方文档与博客平台，这里汇集了核心技术指南、API 参考、最佳实践以及社区动态。我们旨在为开发者提供清晰、准确且持续更新的学习资源，助您快速掌握 MCP 协议，轻松构建连接万物的创新应用。"
    },
    goals: {
      title: "我们的目标",
      items: [
        { icon: "📖", text: "提供易于浏览的项目文档" },
        { icon: "📰", text: "分享项目动态与技术文章" },
        { icon: "💬", text: "支持社区互动" }
      ]
    },
    contact: {
      title: "联系我们",
      items: [
        {
          icon: "fab fa-github",
          title: "GitHub",
          desc: "提交 Issue 或 PR",
          link: "https://github.com/xiangweizeng/mcp-any-rest",
          linkText: "访问仓库"
        },
        {
          icon: "fas fa-envelope",
          title: "邮件联系",
          desc: "商业合作与咨询",
          link: "mailto:support@mcp-any-rest.com",
          linkText: "发送邮件"
        },
        {
          icon: "fab fa-weixin",
          title: "微信交流",
          desc: "加入开发者社群",
          link: null,
          actionText: "ID: MCP-ANY-REST"
        }
      ]
    },
    releases: {
      title: "版本发布"
    }
  },
  "/en/": {
    header: {
      tagline: "A universal protocol layer breaking silos and connecting everything",
      quickStart: "Quick Start"
    },
    about: {
      title: "About MCP-ANY-REST",
      desc: "MCP-ANY-REST is dedicated to building a universal protocol layer that breaks down data silos. As the official documentation and blog platform, this hub gathers core technical guides, API references, best practices, and community updates. We aim to provide clear, accurate, and continuously updated resources to help developers master the MCP protocol and easily build innovative applications that connect everything."
    },
    goals: {
      title: "Our Goals",
      items: [
        { icon: "📖", text: "Provide accessible documentation" },
        { icon: "📰", text: "Share updates & tech articles" },
        { icon: "💬", text: "Support community interaction" }
      ]
    },
    contact: {
      title: "Contact Us",
      items: [
        {
          icon: "fas fa-link",
          title: "GitHub",
          desc: "Submit Issues or PRs",
          link: "https://github.com/xiangweizeng/mcp-any-rest",
          linkText: "Visit Repo"
        },
        {
          icon: "fas fa-envelope",
          title: "Email",
          desc: "Business & Inquiries",
          link: "mailto:support@mcp-any-rest.com",
          linkText: "Send Email"
        },
        {
          icon: "fas fa-message",
          title: "WeChat",
          desc: "Join Community",
          link: null,
          actionText: "ID: MCP-ANY-REST"
        }
      ]
    },
    releases: {
      title: "Release Notes"
    }
  }
};

const t = computed(() => i18n[routeLocale.value] || i18n["/"]);

const parseReleases = (content: string) => {
  const releases = [];
  const lines = content.split('\n');
  let currentRelease: { version: string, date: string, content: string[] } | null = null;

  // Regex to match "### v-0.2.0 2025-11-26" or similar
  const versionRegex = /^###\s+(v-?[\d\.]+)\s+(\d{4}-\d{2}-\d{2})/;

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i].trim();
    const match = line.match(versionRegex);

    if (match) {
      if (currentRelease) {
        releases.push(currentRelease);
      }
      currentRelease = {
        version: match[1],
        date: match[2],
        content: []
      };
    } else if (currentRelease && line.length > 0) {
      // Basic formatter: remove list markers and numbers like "1. " or "- "
      const listMatch = line.match(/^[\d\-\*]+\.?\s+(.*)/);
      if (listMatch) {
        currentRelease.content.push(listMatch[1]);
      } else {
        currentRelease.content.push(line);
      }
    }
  }
  if (currentRelease) {
    releases.push(currentRelease);
  }
  return releases;
};

const releases = computed(() => {
  const content = routeLocale.value === '/en/' ? enReadme : zhReadme;
  return parseReleases(content);
});
</script>

<template>
  <Layout>
    <template #default>
      <div class="about-wrapper">
        <div class="about-header">
          <div class="about-logo">
            <img src="/logo.svg" alt="MCP-ANY-REST Logo" />
          </div>
          <h1>MCP-ANY-REST</h1>
          <p class="tagline">{{ t.header.tagline }}</p>
          <div class="header-actions">
            <a href="https://github.com/xiangweizeng/mcp-any-rest" target="_blank" class="action-btn github">
              <i class="fab fa-github"></i> GitHub
            </a>
            <a href="/guide/quickstart.html" class="action-btn start">
              {{ t.header.quickStart }}
            </a>
          </div>
        </div>

        <div class="about-content">
          <div class="intro-section">
            <h2>{{ t.about.title }}</h2>
            <p>{{ t.about.desc }}</p>
          </div>

          <div class="goals-section">
            <h3>{{ t.goals.title }}</h3>
            <div class="goals-grid">
              <div v-for="(item, index) in t.goals.items" :key="index" class="goal-card">
                <div class="goal-icon">{{ item.icon }}</div>
                <div class="goal-text">{{ item.text }}</div>
              </div>
            </div>
          </div>

          <div class="contact-section">
            <h3>{{ t.contact.title }}</h3>
            <div class="contact-grid">
              <template v-for="(item, index) in t.contact.items" :key="index">
                <a v-if="item.link" :href="item.link" target="_blank" class="contact-card">
                  <div class="card-icon">
                    <i :class="item.icon"></i>
                  </div>
                  <div class="card-content">
                    <h4>{{ item.title }}</h4>
                    <p>{{ item.desc }}</p>
                    <span class="card-action">{{ item.linkText }}</span>
                  </div>
                </a>
                
                <div v-else class="contact-card">
                  <div class="card-icon">
                    <i :class="item.icon"></i>
                  </div>
                  <div class="card-content">
                    <h4>{{ item.title }}</h4>
                    <p>{{ item.desc }}</p>
                    <span class="card-action copy-text">{{ item.actionText }}</span>
                  </div>
                </div>
              </template>
            </div>
          </div>
          
          <!-- Version Release Timeline -->
          <div v-if="releases.length > 0" class="timeline-section">
            <h2>{{ t.releases?.title }}</h2>
            <div class="timeline">
              <div v-for="(release, index) in releases" :key="index" class="timeline-item">
                <div class="timeline-marker"></div>
                <div class="timeline-content">
                  <div class="timeline-header">
                    <span class="version-badge">{{ release.version }}</span>
                    <span class="release-date">{{ release.date }}</span>
                  </div>
                  <ul class="timeline-body">
                    <li v-for="(line, lineIndex) in release.content" :key="lineIndex">{{ line }}</li>
                  </ul>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </template>
  </Layout>
</template>

<style lang="scss">
.about-wrapper {
  max-width: 1100px;
  margin: 0 auto;
  padding: 2rem 1rem;
  box-sizing: border-box;
}


// About Page Styling

.about-header {
  text-align: center;
  padding: 4rem 0 3rem;
  
  .about-logo img {
    height: 180px;
    margin-bottom: 2rem;
    max-height: 200px;
  }
  
  h1 {
    font-size: 3.5rem;
    font-weight: 700;
    margin: 1rem 0;
    background: linear-gradient(120deg, #3b82f6, #8b5cf6);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
  }
  
  .tagline {
    font-size: 1.5rem;
    line-height: 1.6;
    max-width: 800px;
    margin: 1.5rem auto 2.5rem;
    font-weight: 500;
    
    html[data-theme="light"] & {
      color: #334155; // Slate-700
    }
    
    html[data-theme="dark"] & {
      color: #e2e8f0; // Slate-200
      text-shadow: 0 2px 10px rgba(0,0,0,0.5);
    }
  }
  
  .header-actions {
    display: flex;
    gap: 1rem;
    justify-content: center;
    
    .action-btn {
      padding: 0.8rem 2rem;
      border-radius: 2rem;
      text-decoration: none;
      font-weight: 600;
      font-size: 1.1rem;
      transition: all 0.3s ease;
      display: inline-flex;
      align-items: center;
      gap: 0.5rem;
      border: 2px solid transparent;
      
      &:hover {
        transform: translateY(-2px);
      }
      
      &.github {
        background: transparent;
        color: var(--c-text);
        border-color: var(--c-border);
        
        html[data-theme="light"] & {
          color: #334155; // Slate-700
          border-color: #cbd5e1; // Slate-300
          
          &:hover {
            background: #f1f5f9; // Slate-100
            color: #3b82f6;
            border-color: #3b82f6;
          }
        }
        
        html[data-theme="dark"] & {
          color: #e2e8f0; // Slate-200
          border-color: #475569; // Slate-600
          
          &:hover {
            background: rgba(30, 41, 59, 0.6);
            color: #60a5fa; // Blue-400
            border-color: #60a5fa;
          }
        }
      }
      
      &.start {
        background: #3b82f6;
        color: white;
        border-color: #3b82f6;
        box-shadow: 0 4px 12px rgba(59, 130, 246, 0.3);
        
        &:hover {
          background: #2563eb;
          border-color: #2563eb;
          box-shadow: 0 6px 16px rgba(59, 130, 246, 0.4);
        }
      }
    }
  }
}

.about-content {
  margin: 4rem 0;
  
  .intro-section {
    text-align: center;
    max-width: 800px;
    margin: 0 auto 4rem;
    
    h2 {
      font-size: 2rem;
      border-bottom: none;
      margin-bottom: 1.5rem;
      color: #3b82f6;
      
      html[data-theme="dark"] & {
        color: #60a5fa;
      }
    }
    
    p {
      font-size: 1.2rem;
      color: var(--c-text-quote);
      line-height: 1.8;
    }
  }

  .goals-section {
    text-align: center;
    margin-bottom: 4rem;
    
    h3 {
      font-size: 1.5rem;
      margin-bottom: 2rem;
    }
    
    .goals-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
      gap: 1.5rem;
      
      .goal-card {
        background: var(--c-bg-soft);
        padding: 2rem 1.5rem;
        border-radius: 12px;
        transition: transform 0.3s, box-shadow 0.3s, border-color 0.3s;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 1rem;
        border: 1px solid transparent;
        
        html[data-theme="light"] & {
          border-color: #f1f5f9; // Very subtle border default
          box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.05);
        }
        
        html[data-theme="dark"] & {
          background: rgba(30, 41, 59, 0.4); // Darker transparent bg
          border-color: #334155;
        }
        
        &:hover {
          transform: translateY(-5px);
          box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.1), 0 8px 10px -6px rgba(0, 0, 0, 0.1);
          border-color: #3b82f6;
          
          html[data-theme="dark"] & {
             background: rgba(30, 41, 59, 0.8);
          }
        }
        
        .goal-icon {
          font-size: 3rem;
          margin-bottom: 0.5rem;
        }
        
        .goal-text {
          font-size: 1.1rem;
          font-weight: 500;
        }
      }
    }
  }
  
  .contact-section {
    text-align: center;
    padding: 3rem 0;
    
    h3 {
      margin-top: 0;
      margin-bottom: 2rem;
      font-size: 1.5rem;
    }

    .contact-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
      gap: 1.5rem;
      max-width: 1000px;
      margin: 0 auto;
    }
    
    .contact-card {
      background: var(--c-bg-soft);
      padding: 2rem;
      border-radius: 16px;
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 1rem;
      text-decoration: none;
      color: inherit;
      transition: all 0.3s ease;
      border: 1px solid transparent;
      
      html[data-theme="light"] & {
        border-color: #f1f5f9;
        box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.05);
      }
      
      html[data-theme="dark"] & {
        background: rgba(30, 41, 59, 0.4);
        border-color: #334155;
      }
      
      &:hover {
        transform: translateY(-5px);
        border-color: #3b82f6;
        box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.1);
        
        html[data-theme="dark"] & {
          background: rgba(30, 41, 59, 0.8);
          border-color: #60a5fa;
        }
        
        .card-icon {
          color: #3b82f6;
          transform: scale(1.1);
          
          html[data-theme="dark"] & {
            color: #60a5fa;
          }
        }
        
        .card-action {
          color: white;
          background: #3b82f6;
          border-color: #3b82f6;
        }
      }
      
      .card-icon {
        font-size: 2.5rem;
        margin-bottom: 0.5rem;
        transition: all 0.3s ease;
        color: var(--c-text-light);
      }
      
      .card-content {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 0.5rem;
        
        h4 {
          margin: 0;
          font-size: 1.2rem;
          font-weight: 600;
        }
        
        p {
          margin: 0;
          font-size: 0.9rem;
          color: var(--c-text-quote);
          margin-bottom: 1rem;
        }
      }
      
      .card-action {
        padding: 0.5rem 1.5rem;
        border-radius: 20px;
        font-size: 0.9rem;
        font-weight: 500;
        border: 1px solid var(--c-border);
        transition: all 0.3s ease;
        
        &.copy-text {
          cursor: text;
        }
      }
    }
  }
}

.timeline-section {
  max-width: 800px;
  margin: 4rem auto;
  
  h2 {
    text-align: center;
    font-size: 2rem;
    border-bottom: none;
    margin-bottom: 3rem;
    color: #3b82f6;
    
    html[data-theme="dark"] & {
      color: #60a5fa;
    }
  }
  
  .timeline {
    position: relative;
    padding-left: 2rem;
    
    // Vertical line
    &::before {
      content: '';
      position: absolute;
      left: 0;
      top: 0;
      bottom: 0;
      width: 2px;
      background: var(--c-border);
      border-radius: 1px;
    }
  }
  
  .timeline-item {
    position: relative;
    margin-bottom: 2.5rem;
    padding-left: 2rem;
    
    &:last-child {
      margin-bottom: 0;
    }
    
    .timeline-marker {
      position: absolute;
      left: calc(-2rem - 5px);
      top: 0.5rem;
      width: 1rem;
      height: 1rem;
      border-radius: 50%;
      background: var(--c-bg);
      border: 3px solid #3b82f6;
      z-index: 1;
    }
    
    .timeline-content {
      background: var(--c-bg-soft);
      padding: 1.5rem;
      border-radius: 12px;
      border: 1px solid transparent;
      transition: all 0.3s ease;
      
      html[data-theme="light"] & {
        border-color: #f1f5f9;
        box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.05);
      }
      
      html[data-theme="dark"] & {
        background: rgba(30, 41, 59, 0.4);
        border-color: #334155;
      }
      
      &:hover {
        transform: translateX(5px);
        border-color: #3b82f6;
        
        .timeline-marker {
           background: #3b82f6;
           box-shadow: 0 0 0 4px rgba(59, 130, 246, 0.2);
        }
      }
    }
    
    .timeline-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 1rem;
      flex-wrap: wrap;
      gap: 0.5rem;
      
      .version-badge {
        font-size: 1.2rem;
        font-weight: 700;
        color: #3b82f6;
      }
      
      .release-date {
        font-size: 0.9rem;
        color: var(--c-text-quote);
        background: var(--c-bg);
        padding: 0.2rem 0.8rem;
        border-radius: 12px;
      }
    }
    
    .timeline-body {
      margin: 0;
      padding-left: 1.2rem;
      color: var(--c-text);
      
      li {
        margin-bottom: 0.5rem;
        line-height: 1.6;
        
        &:last-child {
          margin-bottom: 0;
        }
      }
    }
  }
}
</style>
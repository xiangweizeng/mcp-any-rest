import { defineUserConfig } from "vuepress"
import { hopeTheme } from "vuepress-theme-hope"
import { viteBundler } from "@vuepress/bundler-vite"
import { mdEnhancePlugin } from "vuepress-plugin-md-enhance"

export default defineUserConfig({
  base: "/mcp-any-rest/",

  locales: {
    "/": {
      lang: "zh-CN",
      title: "MCP-ANY-REST",
      description: "基于 REST 后端的通用 MCP 服务器与 ZML 语言规范",
    },
    "/en/": {
      lang: "en-US",
      title: "MCP-ANY-REST",
      description: "Generic MCP Server based on REST backend & ZML Language Specification",
    },
  },

  head: [
    ["link", { rel: "icon", href: "/mcp-any-rest/favicon.ico" }]
  ],

  bundler: viteBundler(),

  theme: hopeTheme({
    hostname: "https://xiangweizeng.github.io/mcp-any-rest/",

    author: {
      name: "MCP-ANY-REST Team",
      url: "https://github.com/xiangweizeng/",
    },

    logo: "/logo.svg",
    repo: "https://github.com/xiangweizeng/mcp-any-rest",
    docsDir: "web-docs/src",
    darkmode: "toggle",

    blog: {
      medias: {
        GitHub: "https://github.com/xiangweizeng/mcp-any-rest.git",
        QQ: "https://qm.qq.com/q/Aid2Xodszu",
        Email: "weilai.zeng@foxmail.com",
        Gitee: "https://gitee.com/damone/mcp-any-rest.git",
      },
    },

    navbarLayout: {
      start: ["Brand"],
      center: ["Links"],
      end: ["Search", "Repo", "Language", "Outlook"],
    },

    locales: {
      "/": {
        blog: {
          description: "基于 REST 后端的通用 MCP 服务器",
        },
        navbar: [
          { text: "首页", link: "/", icon: "home" },
          { text: "指南", link: "/guide/", icon: "lightbulb" },
          { text: "博客", link: "/blog/", icon: "blog" },
          { text: "反馈", link: "/reviews/", icon: "message" },
          { text: "关于", link: "/about/", icon: "user" },
        ],
        sidebar: {
          "/guide/": [
            {
              text: "指南",
              icon: "lightbulb",
              children: [
                "quickstart",
                "specification",
                "auth-examples",
                "configuration",
                "ide-support",
              ],
            },
          ],
        },
        metaLocales: {
            editLink: "在 GitHub 上编辑此页",
        }
      },
      "/en/": {
        blog: {
          description: "Generic MCP Server based on REST backend",
        },
        navbar: [
          { text: "Home", link: "/en/", icon: "home" },
          { text: "Guide", link: "/en/guide/", icon: "lightbulb" },
          { text: "Blog", link: "/en/blog/", icon: "blog" },
          { text: "Reviews", link: "/en/reviews/", icon: "message" },
          { text: "About", link: "/en/about/", icon: "user" },
        ],
        sidebar: {
          "/en/guide/": [
            {
              text: "Guide",
              icon: "lightbulb",
              children: [
                "quickstart",
                "specification",
                "auth-examples",
                "configuration",
                "ide-support",
              ],
            },
          ],
        },
        metaLocales: {
            editLink: "Edit this page on GitHub",
        }
      },
    },

    markdown: {
      align: true,
      attrs: true,
      codeTabs: true,
      component: true,
      demo: true,
      figure: true,
      gfm: true,
      imgLazyload: true,
      imgSize: true,
      include: true,
      mark: true,
      playground: {
        presets: ["ts", "vue"],
      },
      stylize: [
        {
          matcher: "Recommended",
          replacer: ({ tag }) => {
            if (tag === "em")
              return {
                tag: "Badge",
                attrs: { type: "tip" },
                content: "Recommended",
              };
          },
        },
      ],
      sub: true,
      sup: true,
      tabs: true,
      vPre: true,
    },

    plugins: {
      blog: true,
      search: true,
      icon: {
        assets: "fontawesome",
        prefix: "fas fa-",
      },

      comment: {
        provider: "Giscus",
        repo: "xiangweizeng/mcp-any-rest",
        repoId: "R_kgDOQdVlHg",
        category: "General",
        categoryId: "DIC_kwDOQdVlHs4CzEkU",
      },
    },
  }),
})

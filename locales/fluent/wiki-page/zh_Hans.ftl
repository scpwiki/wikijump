### 维基页面查看

wiki-page-category = 分类：{ $category }

wiki-page-revision = 修改版本：{ $revision }

wiki-page-last-edit = 最后编辑于: { $date } ({ $days ->
  [0] 今日
  [1] 昨日
  *[other] { $days } 日前
})

wiki-page-view-source = 检视源代码

wiki-page-revision-number = 版本编号

wiki-page-revision-created-at = 创建日期

wiki-page-revision-user = 编辑用户

wiki-page-revision-comments = 编辑摘要

wiki-page-revision-rollback = 回滚

wiki-page-revision-type = 类型
  .create = 创建
  .regular = 编辑
  .move = 移动
  .delete = 删除
  .undelete = 恢复

### 维基页面评分

wiki-page-vote-set = 进行评分

wiki-page-vote-remove = 取消评分

wiki-page-vote-list = 查看评分列表

wiki-page-vote-score = 现时评分

### 维基页面编辑

wiki-page-move-new-slug = 新页面网址

wiki-page-layout =
  .default = 预设布局
  .wikidot = Wikidot（旧）
  .wikijump = Wikijump

wiki-page-restore = 选择需恢复的页面

wiki-page-deleted = 于{ $datetime }删除

### 特殊页面

wiki-page-missing = 你访问的页面 //{ $slug }// 并不存在。

    { " *" } [/{ $slug }/edit 创建此页]。

wiki-page-private = + 私有内容

    该网站的这个区域是私人的，您无权访问。如果您认为您的确需要访问该区域，请联系站点管理员。

wiki-page-banned = + 您已经被封禁

    您已经被本站封禁，而本站设定并不允许被封禁用户访问页面。

wiki-page-site-slug = <h1>使用 { -service-name } 作名字的站点并不存在。</h1>
    <p>
      <a href="https://{ $slug }.{ $domain }/">{ $slug }.{ $domain }</a> 并不存在。
      返回 <a href="https://{ $domain }/">{ -service-name }</a>。
    </p>

wiki-page-site-custom = <h1>使用 { -service-name } 作名字的站点并不存在。</h1>
    <p>
      没有站点使用自订域名 <a href="https://{ $custom_domain }/">{ $custom_domain }</a>。
      返回 <a href="https://{ $domain }/">{ -service-name }</a>。
    </p>

wiki-page-no-render = 内容停止显示。

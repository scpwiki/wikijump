### 基本错误 HTML 模板

basic-error-site-slug = <h1>该网址不存在 { -service-name } 站点</h1>
    <p>
      <a href="https://{ $slug }.{ $main_domain }/"><code>{ $slug }.{ $main_domain }</code></a> 不存在。
    </p>

    <p>
      返回 <a href="https://{ $main_domain }/">{ -service-name }</a>。
    </p>

    .title = 站点不存在 - { -service-name }

basic-error-site-custom = <h1>该网址不存在 { -service-name } 站点</h1>
    <p>
      没有网站使用此自定义域名 <a href="https://{ $custom_domain }/"><code>{ $custom_domain }</code></a>。
    </p>

    <p>
      返回 <a href="https://{ $main_domain }/">{ -service-name }</a>。
    </p>

    .title = 站点不存在 - { -service-name }

basic-error-page-slug = <h1>此页面并不存在</h1>
    <p>
      页面 <a href="https://{ $domain }/{ $page_slug }"><code>{ $domain }/{ $page_slug }</code></a> 并不存在。
    </p>

    <p>
      返回<a href="https://{ $domain }/">网站主页面</a>。
    </p>

    .title = 此页面不存在 - { $domain }

basic-error-page-fetch = <h1>无法载入页面</h1>
    <p>
      服务器错误：页面 <a href="https://{ $domain }/{ $page_slug }"><code>{ $domain }/{ $page_slug }</code></a> 无法被载入。
    </p>

    <p>
      返回<a href="https://{ $domain }/{ $page_slug }">页面/a>，或<a href="https://{ $domain }/">网站主页面</a>.
    </p>

    .title = 服务器错误 - { $domain }

basic-error-file-name = <h1>此文件并不存在</h1>
    <p>
      页面 <code>{ $domain }/{ $page_slug }</code> 不存在文件 <code>{ $filename }</code>。
    </p>

    <p>
      返回<a href="https://{ $domain }/{ $page_slug }">页面</a>，或<a href="https://{ $domain }/">网站主页面</a>。
    </p>

    .title = 此页面不存在 - { $domain }

basic-error-file-fetch = <h1>无法载入文件</h1>
    <p>
      服务器错误：位于页面 <code>{ $domain }/{ $page_slug }</code> 的文件 <code>{ $filename }</code> 无法被载入。
    </p>

    <p>
      返回<a href="https://{ $domain }/{ $page_slug }">页面</a>，或<a href="https://{ $domain }/">网站主页面</a>。
    </p>

    .title = 服务器错误 - { $domain }

basic-error-text-block = <h1>无效文字块</h1>
    <p>
      { $reason ->
        [missing] 编号为 <code>{ $index }</code> 的 { $type ->
          [code] 源代码
          [html] HTML
          *[error] 文字
        } 块并不存在。
        [invalid] 无效编号 <code>{ $index }</code> 。
        [fetch] 此文字块无法被载入。
        *[error] 未知基本错误原因：{ $reason }
      }
    </p>

    <p>
      返回<a href="https://{ $domain }/">网站主页面</a>。
    </p>

    .title = 文字块错误 - { $domain }

basic-error-file-root = <h1>无效路径</h1>
    <p>
      { -service-name }  于 <code>{ $files_domain }</code> 提供用户生成的数据，但这不是有效的URL。
    </p>

    <p>
      返回 <a href="https://{ $main_domain }/">{ -service-name }</a>。
    </p>

    .title = { -service-name }

basic-error-blob-fetch = <h1>无法载入文件</h1>
    <p>
      服务器错误：S3 哈希值为 <code>{ $s3_hash }</code> 的文件无法被载入。
    </p>

    .title = 服务器错误

basic-error-user-fetch = <h1>无法载入用户</h1>
    <p>
      服务器错误：编号为 <code>{ $user_id }</code> 的用户无法被载入。
    </p>

    .title = 服务器错误

basic-error-user-avatar = <h1>无法载入用户头像</h1>
    <p>
      服务器错误：编号为 <code>{ $user_id }</code> 的用户头像无法被载入。
    </p>

    .title = 服务器错误

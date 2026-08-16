### 基本错误 HTML 模板

basic-error-site-slug = <h1>该网址不存在 { -service-name } 站点。</h1>
    <p>
      <a href="https://{ $slug }.{ $main_domain }/"><code>{ $slug }.{ $main_domain }</code></a> 不存在。
    </p>

    <p>
      返回 <a href="https://{ $main_domain }/">{ -service-name }</a>。
    </p>

    .title = 站点不存在 - { -service-name }

basic-error-site-custom = <h1>该网址不存在 { -service-name } 站点。</h1>
    <p>
      没有网站使用此自定义域名 <a href="https://{ $custom_domain }/"><code>{ $custom_domain }</code></a>。
    </p>

    <p>
      返回 <a href="https://{ $main_domain }/">{ -service-name }</a>。
    </p>

    .title = 站点不存在 - { -service-name }

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

<script type="text/javascript">
	Wikijump.cancelurl = '/forum/c-{$category->getCategoryId()}/{$category->getUnixifiedName()}';
</script>

<div class="forum-new-thread-box">
	<div class="forum-breadcrumbs">
		<a href="/forum/start">Forum</a>
		&raquo;
		<a href="/forum/c-{$category->getCategoryId()}/{$category->getUnixifiedName()|escape}">{$category->getName()|escape}</a>
		&raquo;
		{t}new thread{/t}
	</div>
	<div class="description">
		<div class="statistics">
			{t}number of threads{/t}: {$category->getNumberThreads()}<br/>
			{t}number of posts{/t}: {$category->getNumberPosts()}
		</div>
		{$category->getDescription() |escape}
	</div>
</div>

<div id="message-preview-wrapper"  style="display: none">
	<h2>{t}Post content preview{/t}:</h2>

	<div id="message-preview" class="thread-container">
	</div>

	{*(<a href="javascript:;" onclick="$('message-preview-wrapper').style.display='none'">close preview</a>)*}
</div>

<div class="error-block" id="new-thread-error" style="display: none"></div>

<div>
	<form id="new-thread-form" action="" onsubmit="return false;">

		<input type="hidden" name="category_id" value="{$category->getCategoryId()}"/>
		<table class="form" style="margin: 1em 0">
			<tr>
				<td>
					{t}Thread title{/t}:
				</td>
				<td>
					<input class="text" type="text" name="title" value="{$title|escape}" size="60" maxlength="99" />
				</td>
			</tr>
			<tr>
				<td>
					{t}Summary{/t}:
				</td>
				<td>
					<textarea cols="60" rows="2" id="thread-description" name="description"></textarea>
					<div class="sub">
						(<span id="desc-charleft"></span> {t}characters left{/t})
					</div>
				</td>
			</tr>
		</table>

		<div id="post-edit-panel" class="wd-editor-toolbar-panel"></div>
		<div><textarea id="post-edit" name="source"  rows="10" style="width: 95%;"></textarea></div>

		<div class="change-textarea-size">
			<a href="javascript:;" onclick="Wikijump.utils.changeTextareaRowNo('post-edit',-5)">-</a>
			<a href="javascript:;" onclick="Wikijump.utils.changeTextareaRowNo('post-edit',5)">+</a>
		</div>
		<div class="edit-help-34">
			{t escape=no}Need help? Check the <a href="{$URL_DOCS}" target="_blank">documentation</a>.{/t}
		</div>
		{if isset($anonymousString)}
			<div class="note-block">
				<h3>Anonymous edit!</h3>
				<p>
					You are starting a new forum discussion thread as an anonymous user.
					Please remember that in such a case your IP address will be revealed to public
					and your contribution will be signed by the following identity:<br/>
					{printuser user=$anonymousString image="true"}
				</p>
			</div>
		{/if}
		<div class="buttons alignleft">
			<input type="button" value="{t}cancel{/t}" id="ntf-cancel"/>
			<input type="button" value="{t}preview{/t}" id="ntf-preview"/>
			<input type="button" value="{t}post{/t}" id="ntf-post"/>
		</div>
	</form>
</div>

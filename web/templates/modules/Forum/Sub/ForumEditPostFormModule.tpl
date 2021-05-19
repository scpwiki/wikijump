<div id="edit-post-preview-div" style="display: none">
	<h1>{t}Preview{/t}:</h1>
	<div></div>
		<a href="javascript:;" onclick="Wikijump.modules.ForumEditPostFormModule.listeners.closePreview(event)">{t}close preview{/t}</a>
</div>

<div id="edit-post-div">
	<form id="edit-post-form">
		<input type="hidden" name="postId" value="{$post->getPostId()}"/>
		<input type="hidden" name="currentRevisionId" value="{$post->getRevisionId()}"/>
		{t}Post title{/t}:<br/>
		<input class="text" style="font-weight:bold; font-size: 130%; width: 95%;" id="np-title" type="text" name="title" size="50" value="{$post->getTitle()|escape}" maxlength="99"/><br/><br/>
		<div id="np-editor-panel" class="wd-editor-toolbar-panel"></div>
		<div><textarea id="np-text" name="source" rows="10" cols="50" style="width: 95%;">{$post->getRevision()->getText()|escape}</textarea></div>
		<div class="change-textarea-size">
			<a href="javascript:;" onclick="Wikijump.utils.changeTextareaRowNo('np-text',-5)">-</a>
			<a href="javascript:;" onclick="Wikijump.utils.changeTextareaRowNo('np-text',5)">+</a>
		</div>
		<div class="edit-help-34">
			{t}Need help? Check the{/t} <a href="{$URL_DOCS}" target="_blank">{t}documentation{/t}</a>.
		</div>
		{if isset($anonymousString)}
			<div class="note-block">
				<h3>{t}Anonymous edit!{/t}</h3>
				<p>
					{t}You are editing a forum post as an anonymous user.
					Please remember that in such a case your IP address will be revealed to public
					and your contribution will be signed by the following identity:{/t}<br/>
					{printuser user=$anonymousString image="true"}
				</p>
			</div>
		{/if}
		<div class="buttons alignleft">
			<input type="button" value="{t}cancel{/t}" id="np-cancel" onclick="Wikijump.modules.ForumEditPostFormModule.listeners.cancel(event)"/>
			<input type="button" value="{t}preview{/t}" id="np-preview" onclick="Wikijump.modules.ForumEditPostFormModule.listeners.preview(event)"/>
			<input type="button" value="{t}save changes{/t}" id="np-post" onclick="Wikijump.modules.ForumEditPostFormModule.listeners.save(event)"/>
		</div>

	</form>
</div>

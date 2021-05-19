<div class="comments-box">
	{if isset($title)}<h1>{$title|escape}</h1>{/if}

	<div class="options" id="comments-options-hidden" {if isset($showComments)}style="display: none"{/if}>
		<a href="javascript:;" onclick="Wikijump.modules.ForumCommentsModule.listeners.showComments(event)">{t}show comments{/t}</a>
	</div>
	<div class="options" id="comments-options-shown" {if !$showComments}style="display: none"{/if}>
		<a href="javascript:;" onclick="Wikijump.modules.ForumCommentsModule.listeners.hideComments(event)">{t}hide all comments{/t}</a>
		| <a href="javascript:;" onclick="Wikijump.modules.ForumViewThreadModule.listeners.unfoldAll(event)">{t}unfold all{/t}</a>
		| <a href="javascript:;" onclick="Wikijump.modules.ForumViewThreadModule.listeners.foldAll(event)">{t}fold all{/t}</a>
	</div>


	<div id="thread-container" class="thread-container" style="margin-top: 1em">
		{if isset($showComments)}
			{module name="Forum/ForumCommentsListModule"}
		{/if}
	</div>



</div>

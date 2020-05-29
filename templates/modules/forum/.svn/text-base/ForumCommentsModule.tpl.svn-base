<div class="comments-box">
	{if $title}<h1>{$title|escape}</h1>{/if}
	
	<div class="options" id="comments-options-hidden" {if $showComments}style="display: none"{/if}>
		<a href="javascript:;" onclick="WIKIDOT.modules.ForumCommentsModule.listeners.showComments(event)">{t}show comments{/t}</a> 
	</div>
	<div class="options" id="comments-options-shown" {if !$showComments}style="display: none"{/if}>
		<a href="javascript:;" onclick="WIKIDOT.modules.ForumCommentsModule.listeners.hideComments(event)">{t}hide all comments{/t}</a> 
		| <a href="javascript:;" onclick="WIKIDOT.modules.ForumViewThreadModule.listeners.unfoldAll(event)">{t}unfold all{/t}</a> 
		| <a href="javascript:;" onclick="WIKIDOT.modules.ForumViewThreadModule.listeners.foldAll(event)">{t}fold all{/t}</a>
	</div>
	
	
	<div id="thread-container" class="thread-container" style="margin-top: 1em">
		{if $showComments}
			{module name="forum/ForumCommentsListModule"}
		{/if}
	</div>
	
	
	
</div>
{capture name="tUrl"}/forum/t-{$thread->getThreadId()}/{$thread->getUnixifiedTitle()|escape}{/capture}
{assign var=tUrl value=$smarty.capture.tUrl}
{assign var=maxNest value=$category->getEffectiveMaxNestLevel()}

{loadmacro set="Forum"}

{assign var=adCount value=0}
{foreach from=$postmap item=postId key=pos}
	{assign var=post value=$posts[$postId]}
	
	{* check if start container *}
	{*{if $pos==0 || $postmap}*}
	{*{if $containerControl[$pos] == 's'}start{/if}*}
	<div class="post-container" id="fpc-{$post->getPostId()}">
		{* POST STARTS *}
		
		{if  $levels[$postId] < $maxNest || $levels[$postId] == $maxNest && ($containerControl[$pos] && ($containerControl[$pos] != 'k'))}
			{assign var=reply value=true}
		{else}
			{assign var=reply value=false}
		{/if}
				
		{macro name="forumpost" post=$post reply=$reply options=true revisionOptions=true headOptions=true}
					
		{* check if close container *}
		{*<ul>
			<li>{$post->getPostId()}</li>
			<li>{$post->getParentId()}</li>
			<li>{$containerControl[$pos]}</li>
		</ul>*}
		{if $containerControl[$pos] != 'k'}
	</div>
					
		{$containerControl[$pos]|replace:'c':'</div>'}
		{/if}
	{assign var=adCount value=$adCount+1}
{/foreach}
		
		
<a href="javascript:;" id="new-post-button" onclick="WIKIDOT.modules.ForumViewThreadModule.listeners.newPost(event,null)">{t}Add a new comment{/t}</a>

	
<div style="display:none" id="post-options-template">
	<a href="javascript:;" onclick="WIKIDOT.modules.ForumViewThreadModule.listeners.showPermalink(event,'%POST_ID%')">{t}permanent link{/t}</a> |
			<a href="javascript:;" onclick="WIKIDOT.modules.ForumViewThreadModule.listeners.editPost(event,'%POST_ID%')">{t}edit{/t}</a> |
			<a href="javascript:;" onclick="WIKIDOT.modules.ForumViewThreadModule.listeners.deletePost(event,'%POST_ID%')">{t}delete{/t}</a>
</div>
	
<script type="text/javascript">
	WIKIDOT.forumThreadId = {$thread->getThreadId()};
</script>

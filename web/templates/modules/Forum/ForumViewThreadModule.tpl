{capture name="tUrl"}/forum/t-{$thread->getThreadId()}/{$thread->getUnixifiedTitle()|escape}{/capture}
{assign var=tUrl value=$smarty.capture.tUrl}
{assign var=maxNest value=$category->getEffectiveMaxNestLevel()}

{loadmacro set="Forum"}



{assign var=wpage value=$thread->getPage()}

<div class="forum-thread-box"> {* one should be able to change the class... *}

	{*{if $wpage}
		<h1><a href="/{$wpage->getUnixName()}">{$wpage->getTitle()|escape}</a> / {t}discussion{/t}</h1>
	{else}
		<h1>{$thread->getTitle()|escape}</h1>
	{/if}*}
		<div  class="forum-breadcrumbs">
			<a href="/forum/start">Forum</a>
			&raquo;
			<a href="/forum/c-{$category->getCategoryId()}/{$category->getUnixifiedName()|escape}">{$category->getForumGroup()->getName()|escape} / {$category->getName()|escape}</a>
			&raquo;
			{$thread->getTitle()|escape}
		</div>


	<div class="description-block">
		<div class="statistics">
			{t}started by{/t}: {printuser user=$thread->getUserOrString() image=true}<br/>
			{t}on:{/t}  <span class="odate">{$thread->getDateStarted()->getTimestamp()}|%e %b %Y, %H:%M %Z|agohover</span><br/>
			{t}number of posts{/t}: {$thread->getNumberPosts()}<br/>
			<span class="rss-icon"><img src="/common--theme/base/images/feed/feed-icon-14x14.png" alt="rss icon"/></span>
			RSS: <a href="/feed/forum/t-{$thread->getThreadId()}.xml">{t}new posts{/t}</a>
		</div>
		{if $wpage == null}
			{if $thread->getDescription()}
				<div class="head">{t}summary{/t}:</div>
				{$thread->getDescription()|escape}
			{/if}
		{else}
			{t}This is the discussion related to the wiki page {/t}
			<a href="/{$wpage->getUnixName()}">{if $wpage->getTitle() && $wpage->getTitle()!=''}{$wpage->getTitle()}{else}{$wpage->getUnixname()}{/if}</a>.
		{/if}

	</div>

	<div class="options">
		<a href="javascript:;" onclick="Wikijump.modules.ForumViewThreadModule.listeners.unfoldAll(event)">{t}unfold all{/t}</a>
		| <a href="javascript:;" onclick="Wikijump.modules.ForumViewThreadModule.listeners.foldAll(event)">{t}fold all{/t}</a>
		| <a href="javascript:;" id="thread-toggle-options" onclick="Wikijump.modules.ForumViewThreadModule.listeners.toggleThreadOptions(event)"> +{t}more options{/t}</a>
	</div>
	<div id="thread-options-2" class="options" style="display: none">
		{if !$wpage}<a href="javascript:;" onclick="Wikijump.modules.ForumViewThreadModule.listeners.editThreadMeta(event)">{t escape=no}edit title &amp; description{/t}</a>
		| <a href="javascript:;" onclick="Wikijump.modules.ForumViewThreadModule.listeners.editThreadStickiness(event)">{t}stickness{/t}</a>
		| {/if}<a href="javascript:;" onclick="Wikijump.modules.ForumViewThreadModule.listeners.editThreadBlock(event)">{t}posting block{/t}</a>
		| <a href="javascript:;" onclick="Wikijump.modules.ForumViewThreadModule.listeners.moveThread(event)">{t}move thread{/t}</a>
		| <a href="javascript:;" onclick="Wikijump.modules.ForumViewThreadModule.listeners.watchThread(event)">{t}add to watched{/t}</a>
	</div>

	<div id="thread-action-area" class="action-area" style="display: none"></div>

	<div id="thread-container" class="thread-container">

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

				{* POST END *}

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

	</div>

	<div class="new-post">
		<a href="javascript:;" id="new-post-button" onclick="Wikijump.modules.ForumViewThreadModule.listeners.newPost(event,null)">{t}new post{/t}</a>
	</div>

	<div style="display:none" id="post-options-template">
			<a href="javascript:;" onclick="Wikijump.modules.ForumViewThreadModule.listeners.showPermalink(event,'%POST_ID%')">{t}permanent link{/t}</a> |
			<a href="javascript:;" onclick="Wikijump.modules.ForumViewThreadModule.listeners.editPost(event,'%POST_ID%')">{t}edit{/t}</a> 
            {if $canDelete}
                 | <a href="javascript:;" onclick="Wikijump.modules.ForumViewThreadModule.listeners.deletePost(event,'%POST_ID%')">{t}delete{/t}</a>
            {/if}
	</div>

	<div style="display:none" id="post-options-permalink-template">{$tUrl}#post-</div>
</div>

<script type="text/javascript">
	Wikijump.forumThreadId = {$thread->getThreadId()};
</script>

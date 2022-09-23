<div class="forum-mini-stat" >
	{foreach from=$posts item=post}
		{assign var=thread value=$post->getForumThread()}
		{assign var=user value=$post->getUser()}
		<div class="item" style="padding-bottom: 5px">
			<div class="title">
				<a href="/forum/t-{$thread->getThreadId()}/{$thread->getUnixifiedTitle()}#post-{$post->getPostId()}">{if $post->getTitle()}{$post->getTitle()|escape}{else}({t}no title{/t}){/if}</a>
			</div>
			<div class="info">
				({t}by{/t} {printuser user=$user} <span class="odate">{$post->getDatePosted()->getTimestamp()}|%O {t}ago{/t}</span>,
				{t}posts{/t}: {$thread->getNumberPosts()})
			</div>
		</div>
	{/foreach}
</div>

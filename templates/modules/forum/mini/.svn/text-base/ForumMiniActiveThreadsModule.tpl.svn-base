<div class="forum-mini-stat">
	{foreach from=$threads item=thread}
		<div class="item">
			<div class="title">
				<a href="/forum/t-{$thread->getThreadId()}/{$thread->getUnixifiedTitle()}">{$thread->getTitle()|escape}</a>
			</div>
			<div class="info">
				({t}started{/t} <span class="odate">{$thread->getDateStarted()->getTimestamp()}|%O {t}ago{/t}</span>,
				{t}posts{/t}: {$thread->getNumberPosts()})
			</div>
		</div>
	{/foreach}
</div>
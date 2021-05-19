<div class="forum-category-box">
	<div class="forum-breadcrumbs">
		<a href="/forum/start">Forum</a>
		&raquo;
		{$category->getForumGroup()->getName()|escape} /
		{$category->getName()|escape}
	</div>

	<div class="description-block">
		<div class="statistics">
			{t}number of threads{/t}: {$category->getNumberThreads()}<br/>
			{t}number of posts{/t}: {$category->getNumberPosts()}<br/>
			<span class="rss-icon"><img src="/common--theme/base/images/feed/feed-icon-14x14.png" alt="rss icon"/></span>
			RSS: <a href="/feed/forum/ct-{$category->getCategoryId()}.xml">{t}new threads{/t}</a> | <a href="/feed/forum/cp-{$category->getCategoryId()}.xml">{t}new posts{/t}</a>

		</div>
		{$category->getDescription() |escape}
	</div>

	{if isset($sortStart)}
		<div class="options">
			{t}order by{/t}: <a href="/forum/c-{$category->getCategoryId()}">{t}last post date{/t}</a> | <strong>{t}thread starting date{/t}</strong>
		</div>
	{else}
		<div class="options">
			{t}order by{/t}: <strong>{t}last post date{/t}</strong> | <a href="/forum/c-{$category->getCategoryId()}/sort/start">{t}thread starting date{/t}</a>
		</div>
	{/if}

	<div class="new-post">
		<a href="/forum:new-thread/c/{$category->getCategoryId()}">{t}create a new thread{/t}</a>
	</div>

	{if isset($sortStart)}
		{capture name="destUrl"}/forum/c-{$category->getCategoryId()}/p/%d/sort/start{/capture}
	{else}
		{capture name="destUrl"}/forum/c-{$category->getCategoryId()}/p/%d{/capture}
	{/if}
	{pager url=$smarty.capture.destUrl total=$pagerData.total_pages known=$pagerData.known_pages current=$pagerData.current_page}


	<table style="width: 98%">
		<tr class="head">
			<td>
				{t}thread name{/t}
			</td>
			<td>
				{t}started{/t}
			</td>
			<td>
				{t}posts{/t}
			</td>
			<td>
				{t}recent post{/t}
			</td>
		</tr>
		{foreach from=$threads item=thread}
			<tr >
				<td class="name">
					<div class="title">
						{if $thread->getSticky()}
							{t}Sticky{/t}:
						{/if}
						<a href="/forum/t-{$thread->getThreadId()}/{$thread->getUnixifiedTitle()|escape}">{$thread->getTitle()|escape}</a><br/>
					</div>
					<div class="description">
						{$thread->getDescription()|escape}
					</div>
				</td>
				<td class="started">
					{t}by{/t}: {printuser user=$thread->getUserOrString() image=true  noip=true} <br/>
					<span class="odate">{$thread->getDateStarted()->getTimestamp()}|%e %b %Y, %H:%M %Z|agohover</span>
				</td>
				<td class="posts">
					{$thread->getNumberPosts()}
				</td>
				<td class="last">
					{if $thread->getNumberPosts()>1 || $thread->getPageId()}
						{assign var=lastPost value=$thread->getLastPost()}
						{if isset($lastPost)}
							{t}by{/t} {printuser user=$lastPost->getUserOrString() image=true noip=true}<br/>
							<span class="odate">{$lastPost->getDatePosted()->getTimestamp()}|(%O ago)</span>
							<a href="/forum/t-{$thread->getThreadId()}/{$thread->getUnixifiedTitle()|escape}#post-{$lastPost->getPostId()}">{ltext lang="en"}jump!{/ltext}{ltext lang="pl"}zobacz!{/ltext}</a>
						{/if}
					{else}
						&nbsp;
					{/if}
				</td>
			</tr>
		{/foreach}
	</table>

	{if $threadsCount > 5}
		{pager url=$smarty.capture.destUrl total=$pagerData.total_pages known=$pagerData.known_pages current=$pagerData.current_page}
	{/if}

</div>

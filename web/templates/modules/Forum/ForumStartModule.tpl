{strip}
<div class="forum-start-box">
	{foreach from=$groups item=group}
		<div class="forum-group" style="width: 98%">
			<div class="head">
				<div class="title">
					{$group->getName()|escape}
				</div>
				<div class="description">
					{$group->getDescription()|escape}
				</div>
			</div>

			{assign var=groupId value=$group->getGroupId()}
			{assign var=categories value=$catarray[$groupId]}
			<div >
				<table>
					<tr class="head">
						<td>
							{t}category name{/t}
						</td>
						<td >
							{t}threads{/t}
						</td>
						<td >
							{t}posts{/t}
						</td>
						<td >
							{t}last post{/t}
						</td>
					</tr>
					{foreach from=$categories item=category}
						<tr>
							<td class="name">
								<div class="title">
									<a href="/forum/c-{$category->getCategoryId()}/{$category->getUnixifiedName()|escape}">{$category->getName()|escape}</a>
								</div>
								<div class="description">
									{$category->getDescription()|escape}
								</div>
							</td>
							<td class="threads">
								{$category->getNumberThreads()|escape}
							</td>
							<td class="posts">
								{$category->getNumberPosts()|escape}
							</td>
							<td class="last">
								{assign var=lastPost value=$category->getLastPost()}
								{if isset($lastPost)}
									{t}by{/t} {printuser user=$lastPost->getUserOrString() image=true noip=true}<br/>
									<span class="odate">{$lastPost->getDatePosted()->getTimestamp()}|(%O ago)</span> <a href="/forum/t-{$lastPost->getThreadId()}#post-{$lastPost->getPostId()}">{t}jump!{/t}</a>
								{else}
									&nbsp;
								{/if}
							</td>
						</tr>
					{/foreach}
				</table>
			</div>
		</div>
	{/foreach}
</div>
<p style="text-align: right">
	{if isset($hidden)}
		<a href="/forum/start/hidden/show">{t}show hidden{/t}</a>
	{else}
		<a href="/forum/start">{t}hide hidden{/t}</a>
	{/if}
</p>
<p style="text-align: right">
	<span class="rss-icon"><img src="/common--theme/base/images/feed/feed-icon-14x14.png" alt="rss icon"/></span> RSS: <a href="/feed/forum/threads.xml">{t}new threads{/t}</a> | <a href="/feed/forum/posts.xml">{t}new posts{/t}</a>
</p>
{/strip}

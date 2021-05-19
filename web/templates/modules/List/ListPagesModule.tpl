<div class="list-pages-box">
	{*{foreach from=$items item=item}
		<div>
			{$item}
		</div>
	{/foreach}*}
	{$itemsContent}

	{if $totalPages>1}
	<div style="text-align: center">
		{pager total=$totalPages current=$currentPage url=$pagerUrl}
	</div>
	{/if}

	{if isset($rssUrl)}
		<div class="feedinfo">
			<span class="rss-icon"><img src="/common--theme/base/images/feed/feed-icon-14x14.png" alt="rss icon"/></span>
			<a href="{$rssUrl}">{t}RSS feed{/t}</a>
		</div>
	{/if}
</div>

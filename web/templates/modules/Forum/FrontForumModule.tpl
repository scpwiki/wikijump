<div class="front-forum-box">
	{foreach from=$items item=item}
		<div>
			{$item}
		</div>
	{/foreach}
	{if isset($feedUri)}
		<div class="feedinfo">
			<span class="rss-icon"><img src="/common--theme/base/images/feed/feed-icon-14x14.png" alt="rss icon"/></span>
			<a href="{$feedUri}">{t}RSS feed{/t}</a>
		</div>
	{/if}
</div>

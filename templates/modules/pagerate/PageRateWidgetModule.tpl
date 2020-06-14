{strip}
{if $type=="P" || $type=="M" || $type=="Z"}
	<div class="page-rate-widget-box">
		<span class="rate-points">{t}rating{/t}:&nbsp;<span class="number" id="prw54353">{if $rate != 0}{$rate|string_format:"%+d"}{else}{$rate}{/if}</span></span>
		<span class="rateup"><a title="{t}Upvote{/t}" href="javascript:;" onclick="WIKIDOT.modules.PageRateWidgetModule.listeners.rate(event, 1)">+</a></span>
		{if $type=="Z" || $type=="X"}
		<span class="ratezero"><a title="{t}No-vote{/t}" href="javascript:;" onclick="WIKIDOT.modules.PageRateWidgetModule.listeners.rate(event, 0)">=</a></span>
		{/if}
		{if $type=="M" || $type=="Z"}
			<span class="ratedown"><a title="{t}Downvote{/t}" href="javascript:;" onclick="WIKIDOT.modules.PageRateWidgetModule.listeners.rate(event, -1)">&#8211;</a></span>
		{/if}
		<span class="cancel"><a title="{t}Cancel my vote{/t}" href="javascript:;" onclick="WIKIDOT.modules.PageRateWidgetModule.listeners.cancelVote(event)">x</a></span>
	</div>
	{elseif $type=="S"}
	<div class="page-rate-widget-box">
		<span class="rate-points">{t}rating{/t}:&nbsp;<span class="number" id="prw54353">{$rate}</span></span>
		<span class="rateup"><a title="{t}Upvote{/t}" href="javascript:;" onclick="WIKIDOT.modules.PageRateWidgetModule.listeners.rate(event, 1)">+</a></span>
		{if $type=="Z" || $type=="X"}
			<span class="ratezero"><a title="{t}No-vote{/t}" href="javascript:;" onclick="WIKIDOT.modules.PageRateWidgetModule.listeners.rate(event, 0)">&empty;</a></span>
		{/if}
		{if $type=="M" || $type=="Z"}
			<span class="ratedown"><a title="{t}Downvote{/t}" href="javascript:;" onclick="WIKIDOT.modules.PageRateWidgetModule.listeners.rate(event, -1)">&#8211;</a></span>
		{/if}
		<span class="cancel"><a title="{t}Cancel my vote{/t}" href="javascript:;" onclick="WIKIDOT.modules.PageRateWidgetModule.listeners.cancelVote(event)">x</a></span>
	</div>
{/if}
{/strip}

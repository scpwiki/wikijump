{strip}
{if $type=="P" || $type=="M"}
	<div class="page-rate-widget-box">
		<span class="rate-points">{t}rating{/t}:&nbsp;<span class="number" id="prw54353">{if $rate != 0}{$rate|string_format:"%+d"}{else}{$rate}{/if}</span></span>
		<span class="rateup"><a title="{t}I like it{/t}" href="javascript:;" onclick="WIKIDOT.modules.PageRateWidgetModule.listeners.rate(event, 1)">+</a></span>
		{if $type=="M"}
			<span class="ratedown"><a title="{t}I don't like it{/t}" href="javascript:;" onclick="WIKIDOT.modules.PageRateWidgetModule.listeners.rate(event, -1)">&#8211;</a></span>
		{/if}
		<span class="cancel"><a title="{t}cancel my vote{/t}" href="javascript:;" onclick="WIKIDOT.modules.PageRateWidgetModule.listeners.cancelVote(event)">x</a></span>
	</div>
{/if}
{/strip}

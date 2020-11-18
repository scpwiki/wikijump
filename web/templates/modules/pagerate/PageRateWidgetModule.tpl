{strip}
{if $type=="P" || $type=="M" || $type=="Z" || $type=="X"}
	<div class="page-rate-widget-box">
		<span class="rate-points">{t}rating{/t}:&nbsp;<span class="number" id="prw54353">{if $rate != 0 && $type != "S"}{$rate|round|string_format:"%+d"}{else}{$rate|round}{/if}</span></span>
		<span class="rateup"><a title="{t}Upvote{/t}" href="javascript:;" onclick="Wikijump.modules.PageRateWidgetModule.listeners.rate(event, 1)">+</a></span>
		{if $type=="Z" || $type=="X"}
		<span class="ratezero"><a title="{t}No-vote{/t}" href="javascript:;" onclick="Wikijump.modules.PageRateWidgetModule.listeners.rate(event, 0)">=</a></span>
		{/if}
		{if $type=="M" || $type=="Z"}
			<span class="ratedown"><a title="{t}Downvote{/t}" href="javascript:;" onclick="Wikijump.modules.PageRateWidgetModule.listeners.rate(event, -1)">&#8211;</a></span>
		{/if}
		<span class="cancel"><a title="{t}Cancel my vote{/t}" href="javascript:;" onclick="Wikijump.modules.PageRateWidgetModule.listeners.cancelVote(event)">x</a></span>
	</div>
	{elseif $type=="S"}
	<div class="page-rate-widget-box">
		<span class="rate-points">{t}rating{/t}:&nbsp;<span class="number" id="prw54353">{$rate}</span></span>
		<span class="one"><a title="{t}1{/t}" href="javascript:;" onclick="Wikijump.modules.PageRateWidgetModule.listeners.rate(event, 1)">1</a></span>
		<span class="two"><a title="{t}2{/t}" href="javascript:;" onclick="Wikijump.modules.PageRateWidgetModule.listeners.rate(event, 2)">2</a></span>
		<span class="three"><a title="{t}3{/t}" href="javascript:;" onclick="Wikijump.modules.PageRateWidgetModule.listeners.rate(event, 3)">3</a></span>
		<span class="four"><a title="{t}4{/t}" href="javascript:;" onclick="Wikijump.modules.PageRateWidgetModule.listeners.rate(event, 4)">4</a></span>
		<span class="five"><a title="{t}5{/t}" href="javascript:;" onclick="Wikijump.modules.PageRateWidgetModule.listeners.rate(event, 5)">5</a></span>
		<span class="cancel"><a title="{t}Cancel my vote{/t}" href="javascript:;" onclick="Wikijump.modules.PageRateWidgetModule.listeners.cancelVote(event)">x</a></span>
	</div>
{/if}
{/strip}

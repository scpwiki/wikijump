<h2>{t}Page rated by:{/t}</h2>

<div style="-moz-column-count:3">
	{foreach from=$rates item=rate}
		{assign var=user value=$rate->getUser()}
		{printuser user=$user image=true}&nbsp;<span style="color:#777">{if $rate->getRate()>0}+{else}-{/if}</span><br/>
	{/foreach}
</div>
results:

{if isset($noData)}
	<p>
		No data for this Member.
	</p>
{else}
	<p>
		{foreach from=$chain item=ring}
			{if $ring.link}
				&rarr; {$ring.link->getTypeDescription()} &rarr;
			{/if}
			{printuser user=$ring.user}
		{/foreach}
	</p>
{/if}

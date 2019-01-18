<p>
	{t}All signatures so far{/t}: <strong>{$campaign->getNumberSignatures()}</strong>
</p>

<p>
	{t}Most recent signatures{/t}:
</p>

{if $signatures}
<ul>
	{foreach from=$signatures item=sig}
		<li>
			{$sig->getFirstName()|escape} {$sig->getLastName()|escape},
			{if $campaign->getCollectCity() && $campaign->getShowCity()}
				{$sig->getCity()|escape},
			{/if}
			{if $campaign->getCollectState() && $campaign->getShowState()}
				{$sig->getState()|escape},
			{/if}
			{if $campaign->getCollectZip() && $campaign->getShowZip()}
				{$sig->getZip()|escape},
			{/if}
			{if $campaign->getCollectCountry() && $campaign->getShowCountry() && $sig->getCountry()}
				{$sig->getCountry()|escape},
			{/if}
			<span  class="odate">{$sig->getDate()->getTimestamp()}|%O ago</span>
			{if $campaign->getCollectComments() && $campaign->getShowComments() && $sig->getComments()}
				<br/>Comments: {$sig->getComments()|escape|nl2br}
			{/if}
			
		</li>
	{/foreach}
</ul>
{/if}
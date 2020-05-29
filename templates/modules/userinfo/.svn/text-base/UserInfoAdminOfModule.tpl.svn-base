<h1>{t}Administrator of the following sites:{/t}</h1>

{if $memberships}
	<div class="sites-list">
		{foreach from=$memberships item=membership}
			{assign var=site value=$membership->getSite()}
			<div class="site-list-item">
				<div class="name">
					<a href="http://{$site->getDomain()}">{$site->getName()|escape}</a>
				</div>
				{if $site->getSubtitle()}
					<div class="subtitle">
						{$site->getSubtitle()|escape}
					</div>
				{/if}
				{if $site->getDescription()}
					<div class="description">
						{$site->getDescription()|escape}
					</div>
				{/if}
			</div>
			
		{/foreach}
	</div>
{else}
	{t}This user is not an administrator of any site.{/t}
{/if}
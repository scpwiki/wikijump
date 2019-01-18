<h1>{t}Applications{/t}</h1>

<p>
	{t}Below is a list of membership applications you have sent.{/t}
</p>

{foreach from=$applications item=application}
	{assign var=site value=$application->getSite()}
	<h3>{t}Application to site{/t} <a href="http://{$site->getDomain()}">{$site->getName()|escape}</a></h3>
	<table class="form alignleft">
		{if $site->getDescription()!= ''}
			<tr>
				<td>
					{t}Site description{/t}:
				</td>
				<td>
					{$site->getDescription()|escape}
				</td>
			</tr>
		{/if}
		{if $application->getComment() != ''}
			<tr>
				<td>
					{t}Your application text{/t}:
				</td>
				<td>
					{$application->getComment()|escape}
				</td>
			</tr>
		{/if}
		<tr>
			<td>
				{t}Application status{/t}:
			</td>
			<td>
				 {$application->getStatus()}
			</td>
		</tr>
		{if $application->getReply() != ''}
			<tr>
				<td>
					{t}Reply text{/t}:
				</td>
				<td>
					{$application->getReply()|escape}
				</td>
			</tr>
		{/if}
		<tr>
			<td>
				{t}Options{/t}:
			</td>
			<td>
				{if $application->getStatus() == 'pending'}
					<a href="javascript:;" onclick="WIKIDOT.modules.AccountApplicationsModule.listeners.remove(event, {$site->getSiteId()}, '{$site->getName()|escape}')">{t}remove application{/t}</a>
				{else}
					<a href="javascript:;" onclick="WIKIDOT.modules.AccountApplicationsModule.listeners.remove2(event, {$site->getSiteId()}, '{$site->getName()|escape}')">{t}remove application{/t}</a>
				{/if}
			</td>
		</tr>
	</table>
{/foreach}

<div id="application-remove-dialog" style="display: none">
	{t}Are you sure you want to remove your membership application for site{/t} <strong>%%SITE_NAME%%</strong>?
</div>
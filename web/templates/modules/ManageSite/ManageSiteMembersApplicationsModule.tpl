<h1>Current Member Applications:</h1>
{if isset($applications)}
	{foreach from=$applications item=application}
		{assign var=user value=$application->getUser()}
		<h3>Membership application from {printuser user=$user image=true}</h3>
		<table class="form alignleft">
			{if $application->getComment() != ''}
				<tr>
					<td>
						Application text:
					</td>
					<td>
						{$application->getComment()|escape}
					</td>
				</tr>
			{/if}
			<tr>
				<td>
					Options:
				</td>
				<td>
					<a href="javascript:;" onclick="Wikijump.modules.ManageSiteMembersApplicationsModule.listeners.accept(event, {$user->id}, '{$user->username}', 'accept')">accept</a>
					or <a href="javascript:;" onclick="Wikijump.modules.ManageSiteMembersApplicationsModule.listeners.accept(event, {$user->id}, '{$user->username}', 'decline')">decline</a>
				</td>
			</tr>
		</table>
	{/foreach}
{else}
	<p>
		There are currently no applications for this site.
	</p>
{/if}


<div id="dialog43" style="display: none">
	You are about to %%TYPE%% membership application from %%USER_NAME%%.
	Feel free to add additional message below:
	<form>
		<textarea cols="20" rows="5" style="width: 95%" id="template-id-stub-app-area"></textarea>
		<div class="sub">
			(<span id="template-id-stub-app-area-left"></span> characters left)
	</form>
</div>

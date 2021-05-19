<h1>Site Moderators</h1>

{if isset($moderators)}
	{foreach from=$moderators item=moderator}
		{assign var=user value=$moderator->getUser()}

		<div>
		<div style="position: absolute; margin-left: 20em">
			<a href="javascript:;" onclick="Wikijump.modules.ManageSiteModeratorsModule.listeners.moderatorPermissions(event, {$moderator->getModeratorId()})" >manage permissions</a>

			| <a href="javascript:;" onclick="Wikijump.modules.ManageSiteModeratorsModule.listeners.removeModerator(event,{$user->id}, '{$user->username}')">remove from moderators</a>
		</div>
		{printuser user=$user image="yes"}
		<div id="mod-permissions-{$moderator->getModeratorId()}" style="display: none"></div>
		</div>
	{/foreach}
{else}
	<p>
		No moderators in this Site.
	</p>
{/if}

<div id="remove-moderator-dialog" style="display: none">
	Are you sure you want to remove <strong>%%USER_NAME%%</strong> from site moderators?
</div>


<form id="sm-mod-perms-form">
	<input type="hidden" name="moderatorId" value="{$moderator->getModeratorId()}"/>
	<table class="form">
		<tr>
			<td>
				Moderate pages:
			</td>
			<td>
				<input class="checkbox" type="checkbox" name="pages" {if $ppages}checked="checked"{/if}/>
				<div class="sub">
					Gets all the privileges related to wiki pages.
				</div>
			</td>
		</tr>
		<tr>
			<td>
				Moderate forum:
			</td>
			<td>
				<input class="checkbox" type="checkbox" name="forum" {if $pforum}checked="checked"{/if}/>
				<div class="sub">
					Gets all the privileges related to forum.
				</div>
			</td>
		</tr>
		{*<tr>
			<td>
				Moderate users:
			</td>
			<td>
				<input class="checkbox" type="checkbox" name="users" {if $pusers}checked="checked"{/if}/>
			</td>
		</tr>*}
	</table>
</form>

<div class="buttons">
	<input type="button" value="close" onclick="WIKIDOT.modules.ManageSiteModeratorsModule.listeners.cancelPermissions(event, {$moderator->getModeratorId()})"/>
	<input type="button" value="save" onclick="WIKIDOT.modules.ManageSiteModeratorsModule.listeners.savePermissions(event, {$moderator->getModeratorId()})"/>
</div>

<form action="javascript:;" onsubmit="WIKIDOT.ManageUsersModule.save(this);">
	<table>
		{foreach from=$users item=user}
			<tr>
				<td>Nick name</td>
				<td><input name="nick_name_{$user.user_id}" class="text" type="text" value="{$user.nick_name}"/></td>
			</tr>
			<tr>
				<td>Set password</td>
				<td><input name="password_{$user.user_id}" class="text" type="text" value=""/></td>
			</tr>
			<tr>
				<td></td>
				<td>
					<input name="mod_{$user.user_id}" id="mod_{$user.user_id}" class="text" type="checkbox" {if $user.mod==1}checked="checked"{/if}/>
					<label for="mod_{$user.user_id}">moderator</label>
				</td>
			</tr>
			<tr>
				<td></td>
				<td>
					<input name="admin_{$user.user_id}" id="admin_{$user.user_id}" class="text" type="checkbox" {if $user.admin==1}checked="checked"{/if}/>
					<label for="admin_{$user.user_id}">administrator</label>
				</td>
			</tr>
			<tr>
				<td colspan="2"><hr/></td>
			</tr>
		{/foreach}
		<tr>
			<td></td>
			<td><input class="button" type="submit" value="save"/></td>
		</tr>
	</table>
</form>

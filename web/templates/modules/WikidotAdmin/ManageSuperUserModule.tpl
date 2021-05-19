{if isset($user)}
	<form action="javascript:;" onsubmit="Wikijump.ManageSuperUserModule.save(this);">
		<input name="key" type="hidden" value="{$key}"/>
		<table>
			<tr>
				<td>Nick name</td>
				<td><input name="nick_name" class="text" type="text" value="{$user.nick_name}"/></td>
			</tr>
			<tr>
				<td>Password</td>
				<td><input name="password1" class="text" type="password" value=""/></td>
			</tr>
			<tr>
				<td>Verify</td>
				<td><input name="password2" class="text" type="password" value=""/></td>
			</tr>
			<tr>
				<td></td>
				<td><input class="button" type="submit" value="save"/></td>
			</tr>
		</table>
	</form>
{/if}

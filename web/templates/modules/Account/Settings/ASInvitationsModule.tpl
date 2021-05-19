<h1><a href="javascript:;" onclick="Wikijump.modules.AccountModule.utils.loadModule('am-settings')">Account settings</a> / Receiving invitations</h1>

<p>
	Site administrators can send individual invitations to other users
	to participate in a particular users community.
	If you want to be able to receive such invitations - check the checkbox.
	If not - uncheck it and the invitations will be blocked for you.
</p>

<form >
	<table class="form">
		<tr>
			<td>
				I want to receive invitations:
			</td>
			<td>
				<input class="checkbox" type="checkbox" id="receive-invitations-ch" {if isset($receiveInvitations)}checked="checked"{/if}/>
			</td>
		</tr>
	</table>
	<div class="buttons">
		<input type="button" value="cancel" onclick="Wikijump.modules.AccountModule.utils.loadModule('am-settings')"/>
		<input type="button" value="save" onclick="Wikijump.modules.ASInvitationsModule.listeners.save(event)"/>
	</div>
</form>

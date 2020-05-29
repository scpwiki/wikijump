<div id="invite-members-module-box">
	<p style="text-align: right">
		<a href="javascript:;" onclick="WIKIDOT.modules.InviteMembersModule.listeners.viewHistory(event)">view the invitations you have sent</a>
	</p>
	<p>
		<a href="javascript:;" onclick="WIKIDOT.modules.InviteMembersModule.listeners.showBulkAdd(event)">+ bulk invitations</a>
	</p>
	
	<div id="invitation-addresses-bulk-box" style="display: none">
		<form>
			<textarea id="invitation-addresses-bulk-text" rows="10" columns="10" style="width: 95%"></textarea>
			<div class="sub">
				Paste the list of recipients in the format:<br/>
				Name &lt;email@example.com&gt;, Another Name &lt;email2@example.com&gt;<br/>
				The algorithm should also handle other formats - just try!<br/>
				Separate entries by a coma or newline.<br/> 
				The adresses you enter here will be <b>added</b> to the list below when you click "process".			
			</div>
			<div class="buttons">
				<input type="button" value="cancel" class="button" onclick="WIKIDOT.modules.InviteMembersModule.listeners.cancelBulkAdd(event)"/>
				<input type="button" value="process" class="button" onclick="WIKIDOT.modules.InviteMembersModule.listeners.processBulkAdd(event)"/>
			</div>
		</form>
		
		<hr/>
	</div>
	
	<form>
	
	
		<div id="invitation-addresses"></div>
		
		
		<p style="text-align: right; padding-right: 20%;">
			to contacts: <a href="javascript:;" onclick="WIKIDOT.modules.InviteMembersModule.listeners.setAllToContacts(event, true)">all</a> | <a href="javascript:;" onclick="WIKIDOT.modules.InviteMembersModule.listeners.setAllToContacts(event, false)">none</a>
		</p>
		<p style="text-align: center;">
			<a href="javascript:;" onclick="WIKIDOT.modules.InviteMembersModule.listeners.moreRecipients(event)">{t}add more recipients{/t}</a>
		</p>
		
		
		<p>
			If you want the invited users to become your contacts and vice versa
			(i.e. add them to your list of contacts and add yourself to their lists) - toggle the "to contacts"
			checkboxes above.
		</p>
		<p>
			You can not send more than <b>20</b> invitations at once.
		</p>
		<h2>{t}What to tell them?{/t}</h2>
		
		<p>
			This is the message your recipient(s) will receive. Please do not use any formatting
			(wiki nor HTML) within your (optional) message. 
		</p>
		
		<hr/>
		<p>
			<b>{t}To{/t}:</b> <span id="recipients-list-formatted" style="color: #009900;"></span>
		</p>
		<p>
			<b>{t}From{/t}:</b> {$user->getNickName()|escape} &lt;{$user->getName()|escape}&gt;
		</p>
		<p>
			<b>{t}Subject{/t}:</b> [{$SERVICE_NAME}] {$user->getNickName()|escape} invites you to join!
		</p>
		
		<p>
			Hello <em>name of the recipient</em>,
		</p>
		<p>
			{$user->getNickName()|escape} {if $profile->getRealName()}({$profile->getRealName()|escape}){/if}
			would like to invite you to join members of the wiki website "{$site->getName()|escape}" created at {$SERVICE_NAME} and
			located at the address http://{$site->getDomain()|escape}.
		</p>
		<textarea cols="30" rows="5" style="width: 90%" id="inv-message"></textarea>
	
		<p>
			Signing up is easy and takes less than a minute. If you already have an account
			at {$SERVICE_NAME}
			you will be able to join the mentioned Site. 
			To proceed or learn more click the follow link:<br/>
			<em>(generated link will be placed here)</em>
		</p>
		<p>
			See you
		</p>
		<p>
			{$user->getNickName()|escape} {if $profile->getRealName()}({$profile->getRealName()|escape}){/if}
		</p>
		
		<div class="buttons">
			<input type="button" value="{t}cancel{/t}" onclick="WIKIDOT.modules.ManagerSiteModule.utils.loadModule('sm-welcome')"/>
			<input type="button" value="{t}send invitations{/t}" onclick="WIKIDOT.modules.InviteMembersModule.listeners.send(event)"/>
		</div>
	
	</form>
	
	<div id="recipient-template" style="display: none">
		<table class="form">
			<tr>
				<td>
					<div class="sub">
						{t}Name (required){/t}:
					</div>
					<input type="text" class="text" size="20" onchange="WIKIDOT.modules.InviteMembersModule.listeners.updateTo(event)"/>
				</td>
				<td>
					<div class="sub">
						{t}Email address{/t}:
					</div>
					<input type="text" class="text" size="20" onchange="WIKIDOT.modules.InviteMembersModule.listeners.updateTo(event)"/>
				</td>
				<td >
					<div class="sub">
						to contacts?
					</div>
					<input type="checkbox" checked="checked"/>
				</td>
				<td style="vertical-align: middle">
					<a href="javascript:;" onclick="WIKIDOT.modules.InviteMembersModule.listeners.removeRecipient(event)">{t}remove{/t}</a>
				</td>
			</tr>	
		</table>
	</div>
</div>
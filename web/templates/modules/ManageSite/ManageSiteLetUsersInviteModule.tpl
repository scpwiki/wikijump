<h1>Let Your Users Invite!</h1>

<p>
	Site Administrators do not have to be the only ones allowed to send invitations
	asking to join this Wiki. You can also allow other Members of this Wiki to send
	such invitations by enabling the option below <b>and</b> placing special modules
	on your Wiki.
</p>

<form>
	<table class="form">
		<tr>
			<td>
				Allow all Members of this Wiki to send invitations?
			</td>
			<td>
				<input type="checkbox" class="checkbox"
					id="sm-allow-users-invite"

					{if isset($enabled)}checked="checked"{/if}/>
			</td>
		</tr>
	</table>
	<div class="buttons">
		<input type="button" value="save" onclick="Wikijump.modules.ManageSiteLetUsersInviteModule.listeners.save(event)"/>
	</div>
</form>

<p>
	You will also need to place the following modules that will provide a nice interface
	for sending invitations:
</p>

<h3><tt>InviteByMembers</tt></h3>

<p>
	This module provides a basic interface for inviting new Members. Just create a new page
	(e.g. <a href="/system:invite"><tt>system:invite</tt></a>) and put the following
	(or similar) code there:
</p>

<div class="code">
	<pre>
Invite your friends to join this Wiki!

[[module SendInvitations]]</pre>
</div>

<p>
	It will also allow your members to view (and delete/resend)
	the invitations they send.
</p>

<h3><tt>WhoInvited</tt></h3>

<p>
	This module will allow your users to look up how particular Members joined this Wiki.
	In particular it will display a chain of invitations. To use it simply create a page, e.g.
	<a href="/system:who-invited"><tt>system:who-invited</tt></a> and put copy/paste/modify
	the following code:
</p>
<div class="code">
	<pre>
Check how particular Members joined this Wiki.

[[module WhoInvited]]
</pre>
</div>

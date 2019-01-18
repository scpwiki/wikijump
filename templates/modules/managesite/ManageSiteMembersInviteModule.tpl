<h1>Invite new members</h1>

<p>
	You can easily invite new members to this Site. Use this option if you want
	to invite a particular User (must already have a Wikidot account). 
	Do not send "spam invitations".
</p>
<p>
	Upon accepting the invitation the User will automatically become a Member of this Site.
</p>

<form id="sm-search-user">
	<table class="form">
		<tr>
			<td>
				User name: 
			</td>
			<td>
				<input class="text" type="text" size="40" id="sm-mi-search-f"/>
				<input class="button" type="button" value="search" id="sm-mi-search-b"/>
			</td>
		</tr>
	</table>
</form>


<div id="sm-mi-search-r" style="margin: 1em 0;">

</div>

<div id="sm-tmp-not" style="display:none">
	<div class="owindow">
		<div class="title">
			Member invitation for user %%USERNAME%%
		</div>
		<div class="content">
			<h1>Invite!</h1>
			<p>
				You are about to invite user <strong>%%USERNAME%%</strong> to become a
				member of this site. You can attach a message to the invitation.
			</p>
			<textarea rows="4" cols="30" style="width: 95%" id="template-id-stub-text"></textarea>
			<div><span id="template-id-stub-charleft">200</span> characters left</div>
		</div>
		
		<div class="button-bar">
			<a href="javascript:;" id="template-id-stub-cancel">cancel</a>
			<a href="javascript:;" id="template-id-stub-send">send invitation</a>
		</div>
	</div>
</div>
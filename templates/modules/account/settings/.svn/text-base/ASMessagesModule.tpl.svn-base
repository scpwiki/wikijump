<h1><a href="javascript:;" onclick="WIKIDOT.modules.AccountModule.utils.loadModule('am-settings')">Account settings</a> / Receiving messages</h1>

<p>
	One of the features of Wikidot is the possibility to send <em>private messages</em>, i.e. 
	direct messages between registered users. It is advised to enable private messages
	but you have a few options here (leave boxes unchecked to block all messages):
</p>

<form id="receive-pl-form">
	<table class="form">
		<tr>
			<td>
				Receive private messages from:
			</td>
			<td>
				<input class="radio" type="radio" name="from" value="a" {if $from == "a"}checked="checked"{/if}/> all registered users<br/>
				<input class="radio" type="radio" name="from" value="mf" {if $from == 'mf'}checked="checked"{/if}/> members of the sites I am also a member of + contacts<br/>
				<input class="radio" type="radio" name="from" value="f" {if $from == 'f'}checked="checked"{/if}/> your contacts only<br/>
				<input class="radio" type="radio" name="from" value="n" {if $from == 'n'}checked="checked"{/if}/> nobody (not recommended)
				
			</td>
		</tr>
	</table>
	<div class="buttons">
		<input type="button" value="cancel" onclick="WIKIDOT.modules.AccountModule.utils.loadModule('am-settings')"/>
		<input type="button" value="save" onclick="WIKIDOT.modules.ASMessagesModule.listeners.save(event)"/>
	</div>
	
</form>

<p>
	We respect your settings but there are situations where they can be overriden. In particular
	following people are allowed to send you messages:
</p>
<ul>
	<li>Administrators of the global wiki farm (they can also contact you by email),</li>
	<li>Moderators and administrators of any of the sites you are a member of.</li>
</ul>

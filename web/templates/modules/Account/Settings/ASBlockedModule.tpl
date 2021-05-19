<h1><a href="javascript:;" onclick="Wikijump.modules.AccountModule.utils.loadModule('am-settings')">Account settings</a> / Blocked users</h1>

<p>
	You can maintain a private list of blocked users. Users from the list
	are not allowed to send you any private message. More options should come soon.
</p>
<p>
	Note: user blocks does not work against administrators of the sited you are a member of.
</p>

<h2>List of blocked users</h2>
{if isset($blocks)}
	<ul style="list-style: none">
		{foreach from=$blocks item=block}
			<li style="margin: 0.2em 0">
				{printuser user=$block->getBlockedUser() image="true"}

				(<a href="javascript:;" onclick="Wikijump.modules.ASBlockedModule.listeners.deleteBlock(event, {$block->getBlockedUserId()}, '{$block->getBlockedUser()->username|escape}')">delete block</a>)

			</li>
		{/foreach}
	</ul>
{else}
there are no blocked users.
{/if}
<div id="show-add-block-button">
	<a href="javascript:;" onclick="Wikijump.modules.ASBlockedModule.listeners.showAddForm(event)">+ add user to blocklist</a>
</div>

<div id="add-block-user-div" style="display: none">
	<h2>Add user to block list</h2>
	<form>
		<table class="form">
			<tr>
				<td>
					User to be blocked:
				</td>
				<td>
					<div id="select-user-div">
						<div class="autocomplete-container" style="width: 20em; padding-top: 3px;">
							<input type="text" id="user-lookup" size="30" class="autocomplete-input text"/>
							<div id="user-lookup-list" class="autocomplete-list"></div>
						</div>
					</div>
					<div id="selected-user-div" style="display: none">
						<span id="selected-user-rendered"></span> (<a href="javascript:;" onclick="Wikijump.modules.ASBlockedModule.listeners.changeUser(event)">change</a>)
					</div>
				</td>
			</tr>
		</table>


		<div class="buttons">
			<input type="button" value="cancel" onclick="Wikijump.modules.ASBlockedModule.listeners.cancelAdd(event)"/>
			<input type="button" value="block user" onclick="Wikijump.modules.ASBlockedModule.listeners.blockUser(event)"/>
		</div>
	</form>
</div>

<h1>{t}My contacts{/t}</h1>

{if $contacts}
	<table class="contact-list-table">
		{foreach from=$contacts item=contact}
			{assign var=user value=$contact}
			<tr>
				<td>
					{printuser user=$user image=true}
				</td>
				<td style="padding-left: 5em">
                    <!-- This used to show email addresses of users. You think you're slick huh? -->
				</td>
				</td>
				<td style="padding-left: 10em">
					<a href="javascript:;" onclick="Wikijump.modules.AccountContactsModule.listeners.removeContact(event, {$user->id})">{t}remove{/t}</a>
				</td>
			</tr>
		{/foreach}
	</table>
{else}
	{t}Sorry, no contacts.{/t}
{/if}


<p id="show-add-contact-button">
	<a href="javascript:;" onclick="Wikijump.modules.AccountContactsModule.listeners.showAddForm(event)">+ {t}add a user to contacts{/t}</a>
</p>

<div id="add-contact-user-div" style="display: none">
	<h2>{t}Add user to the contact list{/t}</h2>
	<form>
		<table class="form">
			<tr>
				<td>
					{t}User to be added{/t}:
				</td>
				<td>
					<div id="select-user-div">
						<div class="autocomplete-container" style="width: 20em; padding-top: 3px;">
							<input type="text" id="user-lookup" size="30" class="autocomplete-input text"/>
							<div id="user-lookup-list" class="autocomplete-list"></div>
						</div>
					</div>
					<div id="selected-user-div" style="display: none">
						<span id="selected-user-rendered"></span> (<a href="javascript:;" onclick="Wikijump.modules.AccountContactsModule.listeners.changeUser(event)">{t}change{/t}</a>)
					</div>
				</td>
			</tr>
		</table>


		<div class="buttons">
			<input type="button" value="{t}cancel{/t}" onclick="Wikijump.modules.AccountContactsModule.listeners.cancelAdd(event)"/>
			<input type="button" value="{t}add to contacts{/t}" onclick="Wikijump.modules.AccountContactsModule.listeners.addContact(event)"/>
		</div>
	</form>
</div>

<p>
	{$countBack} {t}users have you in their contacts lists.{/t}
{if $countBack>0}
	<br/>
	<a href="javascript:;" onclick="Wikijump.modules.AccountContactsModule.listeners.showBack(event)">+ {t}show these users{/t}</a>
{/if}
</p>

<div id="back-contacts-list"></div>

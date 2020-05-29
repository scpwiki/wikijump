<form>
	<table class="form" style="margin: 1em auto 1em 0">
		<tr>
			<td>
				{t}From{/t}:
			</td>
			<td>
				{printuser user=$user image="yes"} ({t}that is you{/t})
			</td>
		</tr>
		<tr>
			<td>
				{t}To{/t}:
			</td>
			<td>
				<div id="selected-user-div" style="display: none">
					<span id="selected-user-rendered"></span> (<a href="javascript:;" onclick="WIKIDOT.modules.PMComposeModule.listeners.changeRecipient(event)">{t}change recipient{/t}</a>)
				</div>
				<div id="select-user-div">
					{t}Type the Wikidot User name below or{/t} <a href="javascript:;" onclick="WIKIDOT.modules.PMComposeModule.listeners.showContactsList(event)">{t}select from your contacts{/t}</a>  <br/>
					<div class="autocomplete-container" style="width: 20em; padding-top: 3px;">
						<input type="text" id="user-lookup" size="30" class="autocomplete-input text"/>
						<div id="user-lookup-list" class="autocomplete-list"></div>
					</div>
				</div>
			</td>
		</tr>
		<tr>
			<td>
				{t}Subject{/t}:
			</td>
			<td>
				<input class="text" type="text" id="pm-subject" size="50" maxlength="60" style="font-weight: bold"
					value="{$subject|escape}"/>
			</td>
		</tr>
	</table>
	<hr/>
	<div id="editor-panel" class="wd-editor-toolbar-panel"></div>
	<div><textarea id="editor-textarea" cols="40" rows="15" style="width: 95%">{$body|escape}</textarea></div>
	
	<div class="change-textarea-size">
		<a href="javascript:;" onclick="WIKIDOT.utils.changeTextareaRowNo('editor-textarea',-5)">-</a>
		<a href="javascript:;" onclick="WIKIDOT.utils.changeTextareaRowNo('editor-textarea',5)">+</a>
	</div>
	<div class="edit-help-34">
		{t}Need help? Check the{/t} <a href="{$URL_DOCS}" target="_blank">{t}documentation{/t}</a>.
	</div>
	<div class="buttons alignleft">
		<input type="button" value="{t}cancel{/t}" id="pm-compose-cancel-button"/>
		<input type="button" value="{t}save as draft{/t}" onclick="WIKIDOT.modules.PMComposeModule.listeners.saveDraft(event)"/>
		<input type="button" value="{t}preview{/t}" onclick="WIKIDOT.modules.PMComposeModule.listeners.preview(event)"/>
		<input type="button" value="{t}send!{/t}" onclick="WIKIDOT.modules.PMComposeModule.listeners.send(event)"/>
	</div>
</form>

<div id="pm-preview-area"></div>
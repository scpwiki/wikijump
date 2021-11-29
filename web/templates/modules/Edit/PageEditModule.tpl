{if $newPage}
	<h1>{t}Create a new page{/t}</h1>
{else}
	<h1>{t}Edit the page{/t}</h1>
{/if}

{if $lock}

{else}
	<div>

		<form id="edit-page-form"{if $form} class="edit-with-form"{/if}>
			<input type="hidden" name="page_id" value="{$pageId|escape}"/>
			<table class="form" style="margin: 0.5em auto 1em 0">
				<tr>
					<td>
						{t}Title of the page{/t}:
					</td>
					<td>
						<input class="text" id="edit-page-title" name="title" type="text" value="{$title|escape}" size="35" maxlength="128"
							style="font-weight: bold; font-size: 130%;"/>
					</td>
				</tr>
				{if $newPage && $templates}
					<tr>
						<td>
							{t}Initial content{/t}:
						</td>
						<td>
							<select name="theme" id="page-templates" onchange="Wikijump.modules.PageEditModule.listeners.templateChange(event)">
								<option value=""  style="padding: 0 1em">no template (blank page)</option>
								{foreach from=$templates item=template}
									<option value="{$template->getPageId()}"  style="padding: 0 1em" {if $template->getPageId() == $templateId}selected="selected"{/if}>{$template->getTitle()|escape}</option>
								{/foreach}
							</select>
						</td>
					</tr>
				{/if}
			</table>
            {if $form}
                <input type="hidden" name="form" value="true"/>
                <table class="form" style="margin: 0.5em auto 1em 0pt">
                    {foreach from=$form->fields item=field}
                        <tr>
                            <td>{$field.label|escape}: </td>
                            <td>{$field.editor->renderEdit()}</td>
                        </tr>
                    {/foreach}
                </table>
            {else}
                <div class="wd-editor-toolbar-panel" id="wd-editor-toolbar-panel"></div>
                <div>
                    <textarea id="edit-page-textarea" name="source" rows="20" cols="40" style="width: 95%;">{$source|escape}</textarea>
                </div>
                <div class="change-textarea-size">
                    <a href="javascript:;" onclick="Wikijump.utils.changeTextareaRowNo('edit-page-textarea',-5)">-</a>
                    <a href="javascript:;" onclick="Wikijump.utils.changeTextareaRowNo('edit-page-textarea',5)">+</a>
                </div>
                <div class="edit-help-34">
                    {t}Need help? Check the{/t} <a href="{$URL_DOCS}" target="_blank">{t}documentation{/t}</a>.
                </div>
            {/if}

			<table style="padding: 2px 0; border: none;">
				<tr>
					<td style="border: none; padding: 0 5px;">
						<div >
							{t}Short description of changes{/t}:
							<br/>
							<textarea id="edit-page-comments" name="comments" rows="3" cols="40" ></textarea>
						</div>
						<div class="sub">
							{t escape=no}max 200 characters (<span id="comments-charleft"></span> left){/t}
						</div>
					</td>
					<td style="border: none; padding: 0 5px;">
						<div id="lock-info" {if $disableLocks}style="display: none"{/if}>
							{t 1="900" escape=no}You have acquired an exclusive 15-minute page lock which means nobody else can edit the page simultaneously to
								avoid conflicts.
								<br/>
								The lock will expire in <strong><span id="lock-timer">%1</span></strong> seconds of inactivity.{/t}
						</div>
					</td>
				</tr>
			</table>

			{if $anonymousString}
				<div class="note-block">
					<h3>{t}Anonymous edit!{/t}</h3>
					<p>
						{t}You are editing this page content as an anonymous user.
						Please remember that in such a case your IP address will be revealed to public
						and the changes will be signed by the following identity:{/t}<br/>
						{printuser user=$anonymousString image="true"}
					</p>
				</div>
			{/if}

			<div class="buttons alignleft">
				<input type="button" name="cancel" id="edit-cancel-button" value="{t}cancel{/t}" onclick="Wikijump.modules.PageEditModule.listeners.cancel(event)"/>
				{if !$newPage}<input type="button" name="diff" id="edit-diff-button" value="{t}view diff{/t}" onclick="Wikijump.modules.PageEditModule.listeners.viewDiff(event)"/>{/if}
				<input type="button" name="preview" id="edit-preview-button" value="{t}preview{/t}" onclick="Wikijump.modules.PageEditModule.listeners.preview(event)"/>
				{if !$newPage}<input type="button" name="save-continue" id="edit-save-continue-button"  value="{t escape=no}save &amp; continue{/t}" onclick="Wikijump.modules.PageEditModule.listeners.saveAndContinue(event)"/>{/if}
				<input type="button" name="save" id="edit-save-button"  value="{t}save{/t}" onclick="Wikijump.modules.PageEditModule.listeners.save(event)"/>
			</div>
		</form>


	</div>

	<div id="view-diff-div"></div>

	<div id="preview-message" style="display: none">
		<div class="preview-message">
			{t}This is a preview only!!!{/t}<br/>
			{t}If you leave this page now, all the changes will be lost.{/t}<br/>
			<a href="javascript:;" onclick="OZONE.visuals.scrollTo('action-area')">{t}down to edit{/t}</a> |
			<a href="javascript:;" onclick="document.getElementById('action-area-top').innerHTML=''">{t}close this box{/t}</a>
		</div>
	</div>
{/if}

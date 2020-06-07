<?php /* Smarty version 2.6.7, created on 2008-12-06 20:41:24
         compiled from /var/www/wikidot/templates/modules/rename/RenamePageModule.tpl */ ?>
<?php require_once(SMARTY_CORE_DIR . 'core.load_plugins.php');
smarty_core_load_plugins(array('plugins' => array(array('block', 't', '/var/www/wikidot/templates/modules/rename/RenamePageModule.tpl', 2, false),)), $this); ?>
<?php if (! $this->_tpl_vars['delete']): ?>
	<h1><?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Rename/move page<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?></h1>

	<p>
		<?php $this->_tag_stack[] = array('t', array('escape' => false)); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>The <em>rename</em> action will change the "unix name" of the page, i.e. the address
		via which the page is accessed.<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
	</p>
<?php else: ?>
	<h1><?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Delete page<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?></h1>
	<?php if ($this->_tpl_vars['canDelete']): ?>
		<p>
			<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>
				You can delete the page by either move it to a "deleted" category or by just removing it
				from the Wiki - which is nonrecoverable and should be used with caution.
			<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
		</p>
		<table class="form">
			<tr>
				<td>
					<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>What to do?<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
				</td>
				<td>
					<input type="radio" name="how" value="rename" checked="checked" onclick="$('rename-option-delete').style.display='none';$('rename-option-rename').style.display='block'; OZONE.visuals.scrollTo('rename-option-rename');"> <?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>just rename<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
					<br/>
					<input type="radio" name="how" value="delete" onclick="$('rename-option-delete').style.display='block';$('rename-option-rename').style.display='none'"> <?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>delete completely<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
				</td>
			</tr>
		</table>
	<?php endif;  endif; ?>

<div id="rename-option-rename">
	<?php if ($this->_tpl_vars['delete']): ?>
		<p>
			<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>By preceding the page name with "deleted:" it can be moved to a different category (namespace).
			It is more or less equivalent to delete but no information is lost.<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
		</p>
	<?php endif; ?>


	<?php if ($this->_tpl_vars['isForum']): ?>
		<div class="warning-block">
			<div class="title">Warning!</div>
			<p>
				This page might be important for proper functioning of the discussion forum.
			</p>
			<p>
				By renaming it, editing or deleting you could simply mess it up. These actions
				do not operate on forum elements such as threads, posts etc. but rather
				on particular Wiki pages that contain elements responsible for displaying
				forum elements.
			</p>
			<p>
				<b>Proceed only if you know what you are doing.</b>
			</p>
		</div>
	<?php endif; ?>
	<?php if ($this->_tpl_vars['isAdmin']): ?>
		<div class="warning-block">
			<div class="title">Warning!</div>
			<p>
				This page might be important for proper functioning of the administrative stuff and
				most probably contains modules that allow you configuring and managing this Wiki.
			</p>
			<p>
				<b>Proceed only if you know what you are doing.</b>
			</p>
		</div>
	<?php endif; ?>

	<p>
		<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Attention should be also paid to the pages that depend on this one either by directly linking to
		it or by including it. Click below to see what pages depend on this one.<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
	</p>
	<p>
		<a id="rename-show-backlinks" href="javascript:;" onclick="WIKIDOT.modules.RenamePageModule.listeners.showBacklinks(event)">+ <?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>show dependencies<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?></a>
		<a id="rename-hide-backlinks" style="display:none" href="javascript:;" onclick="WIKIDOT.modules.RenamePageModule.listeners.hideBacklinks(event)">- <?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>hide dependencies<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?></a>
	</p>

	<div id="rename-backlinks-box" style="display:none"></div>

	<div id="rename-error-block" class="error-block" style="display: none"></div>
	<form onsubmit="WIKIDOT.modules.RenamePageModule.listeners.rename(event); return false;">
		<table class="form">
			<tr>
				<td>
					<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Current page name<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>:
				</td>
				<td>
					<?php echo $this->_tpl_vars['page']->getUnixName(); ?>

				</td>
			</tr>
			<tr>
				<td>
					<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>New page name<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>:
				</td>
				<td>
					<input class="text" type="text" id="move-new-page-name" value="<?php echo $this->_tpl_vars['newName']; ?>
" size="30"/>
				</td>
			</tr>
		</table>
		<div class="buttons">
			<input type="button" value="<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>cancel<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>" onclick="WIKIDOT.page.listeners.closeActionArea(event)"/>
			<input type="button" value="<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>rename/move<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>" onclick="WIKIDOT.modules.RenamePageModule.listeners.rename(event)"/>
		</div>
	</form>
</div>

<div id="rename-option-delete" style="display: none">
	<p>
		This will remove the page completely and it will not be possible to recover the data. Are
		you sure you want to do this?
	</p>
	<form>
		<div class="buttons">
			<input type="button" value="<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>cancel<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>" onclick="WIKIDOT.page.listeners.closeActionArea(event)"/>
			<input type="button" value="<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>delete<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>" onclick="WIKIDOT.modules.RenamePageModule.listeners.deletePage(event)"/>
		</div>
	</form>
</div>
import AppKit

extension AppDelegate {
    func configureMainMenu() {
        let mainMenu = NSMenu()

        let appMenuItem = NSMenuItem()
        let appMenu = NSMenu(title: "Foreman")
        appMenu.addItem(targetedMenuItem("About Foreman", action: #selector(showAbout), key: ""))
        appMenu.addItem(.separator())
        appMenu.addItem(targetedMenuItem("Settings…", action: #selector(openSettings), key: ","))
        appMenu.addItem(.separator())
        appMenu.addItem(NSMenuItem(title: "Hide Foreman", action: #selector(NSApplication.hide(_:)), keyEquivalent: "h"))
        let hideOthers = NSMenuItem(title: "Hide Others", action: #selector(NSApplication.hideOtherApplications(_:)), keyEquivalent: "h")
        hideOthers.keyEquivalentModifierMask = [.command, .option]
        appMenu.addItem(hideOthers)
        appMenu.addItem(NSMenuItem(title: "Show All", action: #selector(NSApplication.unhideAllApplications(_:)), keyEquivalent: ""))
        appMenu.addItem(.separator())
        appMenu.addItem(targetedMenuItem("Quit Foreman", action: #selector(quit), key: "q"))
        appMenuItem.submenu = appMenu
        mainMenu.addItem(appMenuItem)

        let foremanMenuItem = NSMenuItem()
        let foremanMenu = NSMenu(title: "Foreman")
        foremanMenu.addItem(targetedMenuItem("Open Foreman", action: #selector(openOverlay), key: "o"))
        refreshMenuItem = targetedMenuItem("Refresh Agents", action: #selector(refresh), key: "r")
        foremanMenu.addItem(refreshMenuItem!)
        foremanMenu.addItem(targetedMenuItem("Flash Jump", action: #selector(beginFlashJump), key: "j"))
        foremanMenu.addItem(targetedMenuItem("Cycle Theme", action: #selector(cycleTheme), key: "t"))
        foremanMenu.addItem(targetedMenuItem("Settings…", action: #selector(openSettings), key: ","))
        foremanMenuItem.submenu = foremanMenu
        mainMenu.addItem(foremanMenuItem)

        NSApp.mainMenu = mainMenu
    }

    func makeStatusMenu() -> NSMenu {
        let menu = NSMenu()
        menu.addItem(targetedMenuItem("Open Foreman", action: #selector(openOverlay), key: "o"))
        menu.addItem(targetedMenuItem("Refresh Agents", action: #selector(refresh), key: "r"))
        menu.addItem(targetedMenuItem("Flash Jump", action: #selector(beginFlashJump), key: "j"))
        menu.addItem(targetedMenuItem("Cycle Theme", action: #selector(cycleTheme), key: "t"))
        menu.addItem(targetedMenuItem("Settings…", action: #selector(openSettings), key: ","))
        menu.addItem(.separator())
        menu.addItem(targetedMenuItem("Quit Foreman", action: #selector(quit), key: "q"))
        return menu
    }

    func targetedMenuItem(_ title: String, action: Selector, key: String) -> NSMenuItem {
        let item = NSMenuItem(title: title, action: action, keyEquivalent: key)
        item.target = self
        return item
    }

}

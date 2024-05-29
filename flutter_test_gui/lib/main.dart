import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test_gui/src/rust/frb_generated.dart';
import 'package:bitsdojo_window/bitsdojo_window.dart';
import 'package:flutter_test_gui/pages/settings_screen.dart';
import 'package:flutter_test_gui/pages/send_screen.dart';
import 'package:flutter_test_gui/pages/receive_screen.dart';
import 'package:flutter_test_gui/consts/consts.dart';

/// Main entrypoint of the application.
///
/// This function is called when the application starts. It initializes the
/// Rust library, sets up the application widget, and shows the window.
///
/// The function first calls the [RustLib.init] function to initialize the
/// Rust library. Then, it runs the application using the [runApp] function
/// with the [MyApp] widget. If the application is running on Windows, Linux,
/// or macOS, it sets up the window properties such as the minimum size,
/// initial size, alignment, and title. Finally, it shows the window.
Future<void> main() async {
  // Initialize the Rust library
  await RustLib.init();

  // Set up the application widget
  runApp(const MyApp());

  // Set up the window properties if running on Windows, Linux, or macOS
  if (Platform.isWindows || Platform.isLinux || Platform.isMacOS) {
    doWhenWindowReady(() {
      final win = appWindow;

      // Set the minimum size of the window
      const initialSize = Size(720, 512);
      win.minSize = initialSize;

      // Set the initial size of the window
      win.size = initialSize;

      // Set the alignment of the window
      win.alignment = Alignment.center;

      // Set the title of the window
      win.title = 'Caesar Test Demo';

      // Show the window
      win.show();
    });
  }
}

/// The root widget of the application.
///
/// It sets up the material design theme and provides the home page.
class MyApp extends StatefulWidget {
  /// Creates a new instance of [MyApp].
  const MyApp({super.key});

  @override
  State<MyApp> createState() => _MyAppState();
}

/// The state for the [MyApp] widget.
class _MyAppState extends State<MyApp> {
  @override
  Widget build(BuildContext context) {
    // Set up the material design theme.
    return MaterialApp(
      title: 'Caesar-Transfer',
      theme: ThemeData(
        useMaterial3: true,
      ),
      // Set the home page.
      home: const MyHomePage(
        title: 'Caesar-Transfer',
      ),
    );
  }
}

/// The root widget of the application that represents the home page.
///
/// It sets up the material design theme and provides the home page.
class MyHomePage extends StatefulWidget {
  /// Creates a new instance of [MyHomePage].
  ///
  /// The [title] argument is the title of the home page.
  const MyHomePage({
    super.key,
    required this.title,
  });

  /// The title of the home page.
  final String title;

  @override
  State<MyHomePage> createState() => _MyHomePageState();
}

/// The state for the [MyHomePage] widget.
class _MyHomePageState extends State<MyHomePage> {
  /// The list of screens that can be displayed on the home page.
  final List<Widget> _screens = [
    SendScreen(),
    ReceiveScreen(),
  ];

  /// The index of the currently selected screen.
  int _selectedIndex = 0;

  /// Handles the tap event on a tab.
  ///
  /// Updates the selected index and rebuilds the widget tree.
  ///
  /// The [index] argument is the index of the selected tab.
  void _onItemTapped(int index) {
    setState(() {
      _selectedIndex = index;
    });
  }

  /// Builds the user interface for the home page.
  ///
  /// It creates a [MaterialApp] widget with a [Scaffold] as the home page.
  /// The [Scaffold] includes an [AppBar], [BottomNavigationBar], and a [Body].
  /// The [AppBar] displays the title of the app and a [PopupMenuButton] for
  /// accessing the settings screen. The [BottomNavigationBar] displays icons
  /// for the send and receive screens and allows the user to select one of them.
  /// The [Body] displays the currently selected screen.
  ///
  /// Returns a [Widget] that represents the user interface for the home page.
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      debugShowCheckedModeBanner: false,
      home: Scaffold(
        // Sets the background color of the scaffold.
        backgroundColor: Constants.backColor,
        appBar: AppBar(
          // Sets the background color of the app bar.
          backgroundColor: const Color(0xFF292c3c), //0xFF282C34),
          centerTitle: true,
          // Sets the title of the app bar.
          title: Text(
            widget.title,
            // Sets the style of the title text.
            style: const TextStyle(color: Constants.textColor),
          ),
          // Sets the action buttons for the app bar.
          actions: [
            PopupMenuButton<String>(
              // Sets the action to perform when a menu item is selected.
              onSelected: (String result) {
                if (result == 'Settings') {
                  // Navigates to the settings screen when the 'Settings' menu item is selected.
                  Navigator.push(
                    context,
                    MaterialPageRoute(builder: (context) => SettingsScreen()),
                  );
                }
              },
              // Sets the items to display in the popup menu.
              itemBuilder: (BuildContext context) => <PopupMenuEntry<String>>[
                const PopupMenuItem<String>(
                  // Sets the value and label of a menu item.
                  value: 'Settings',
                  child: Text('Settings'),
                )
              ],
            ),
          ],
        ),
        // Sets the body of the scaffold.
        body: _screens[_selectedIndex],
        // Sets the bottom navigation bar.
        bottomNavigationBar: BottomNavigationBar(
          // Sets the background color of the bottom navigation bar.
          backgroundColor: const Color(0xFF292c3c), //0xFF282C34),
          // Sets the items to display in the bottom navigation bar.
          items: const <BottomNavigationBarItem>[
            BottomNavigationBarItem(
              // Sets the icon and label of a bottom navigation bar item.
              icon: Icon(Icons.send),
              label: 'Send',
            ),
            BottomNavigationBarItem(
              icon: Icon(Icons.download),
              label: 'Receive',
            ),
          ],
          // Sets the currently selected item in the bottom navigation bar.
          currentIndex: _selectedIndex,
          selectedItemColor: Constants.highlightColor,
          unselectedItemColor: Constants.textColor,
          // Sets the action to perform when an item is tapped.
          onTap: _onItemTapped,
        ),
      ),
    );
  }
}

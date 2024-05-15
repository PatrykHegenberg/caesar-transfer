import 'dart:io';

import 'package:caesar_transfer/pages/settings_screen.dart';
import 'package:flutter/material.dart';
import 'package:caesar_transfer/pages/send_screen.dart';
import 'package:caesar_transfer/pages/receive_screen.dart';
import './messages/generated.dart';
import 'package:bitsdojo_window/bitsdojo_window.dart';

void main() async {
  await initializeRust();
  runApp(const MyApp());
  if (Platform.isWindows || Platform.isLinux || Platform.isMacOS) {
    doWhenWindowReady(() {
      final win = appWindow;
      const initialSize = Size(720, 512);
      win.minSize = initialSize;
      win.size = initialSize;
      win.alignment = Alignment.center;
      win.title = 'Caesar Test Demo';
      win.show();
    });
  }
}

const backColor = Color(0xFF32363E);
const highlightColor = Color(0xFF98C379);
const textColor = Color(0xFFABB2BF);

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  // This widget is the root of your application.
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Caesar-Transfer',
      theme: ThemeData(
        useMaterial3: true,
      ),
      home: const MyHomePage(title: 'Caesar-Transfer'),
    );
  }
}

class MyHomePage extends StatefulWidget {
  const MyHomePage({super.key, required this.title});

  final String title;

  @override
  State<MyHomePage> createState() => _MyHomePageState();
}

class _MyHomePageState extends State<MyHomePage> {
  final List<Widget> _screens = [
    SendScreen(),
    ReceiveScreen(),
  ];
  int _selectedIndex = 0;
  void _onItemTapped(int index) {
    setState(() {
      _selectedIndex = index;
    });
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      debugShowCheckedModeBanner: false,
      home: Scaffold(
        backgroundColor: backColor,
        appBar: AppBar(
          backgroundColor: const Color(0xFF282C34),
          centerTitle: true,
          title: Text(
            widget.title,
            style: TextStyle(color: textColor),
          ),
          actions: [
            PopupMenuButton<String>(
              onSelected: (String result) {
                if (result == 'Settings') {
                  Navigator.push(
                    context,
                    MaterialPageRoute(builder: (context) => SettingsScreen()),
                  );
                }
              },
              itemBuilder: (BuildContext context) => <PopupMenuEntry<String>>[
                const PopupMenuItem<String>(
                  value: 'Settings',
                  child: Text('Settings'),
                )
              ],
            ),
          ],
        ),
        body: _screens[_selectedIndex],
        bottomNavigationBar: BottomNavigationBar(
          backgroundColor: const Color(0xFF282C34),
          items: const <BottomNavigationBarItem>[
            BottomNavigationBarItem(
              icon: Icon(Icons.send),
              label: 'Send',
            ),
            BottomNavigationBarItem(
              icon: Icon(Icons.download),
              label: 'Receive',
            ),
          ],
          currentIndex: _selectedIndex,
          selectedItemColor: highlightColor,
          unselectedItemColor: textColor,
          onTap: _onItemTapped,
        ),
      ),
    );
  }
}
